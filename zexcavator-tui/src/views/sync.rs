use std::num::NonZero;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bip0039::{English, Mnemonic};
use http::Uri;
use pepper_sync::sync::{SyncConfig, TransparentAddressDiscovery};
use pepper_sync::sync_status;
use pepper_sync::wallet::OutputInterface;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::{Application, Frame, NoUserEvent};
use zingolib::config::{load_clientconfig, ChainType, DEFAULT_LIGHTWALLETD_SERVER};
use zingolib::data::PollReport;
use zingolib::lightclient::{self, LightClient};
use zingolib::wallet::{LightWallet, WalletBase, WalletSettings};

use crate::components::log_viewer::LogBuffer;
use crate::components::sync_bar::SyncBar;
use crate::walletparsers::walletparsers::WalletParserFactory;
use crate::{Id, Msg};

use super::{Mountable, Renderable};

#[derive(Debug, Clone)]
pub struct SyncView {
    log_buffer: LogBuffer,
    // Progress between 0.0 and 1.0
    pub progress: Arc<Mutex<f32>>,
    pub sync_complete: Arc<Mutex<bool>>, // TODO: Replace with AtomicBool
}

impl SyncView {
    pub fn new_with_log(log_buffer: LogBuffer) -> Self {
        Self {
            log_buffer,
            progress: Arc::new(Mutex::new(0.0)),
            sync_complete: Arc::new(Mutex::new(false)),
        }
    }

    pub fn get_progress(&self) -> Arc<Mutex<f32>> {
        Arc::clone(&self.progress)
    }

    pub async fn start_wallet_sync_from_path(&self, path: PathBuf) -> LightClient {
        let wallet_parser = WalletParserFactory::read(path.to_str().unwrap()).unwrap();

        let seed = wallet_parser.parser.get_wallet_seed();
        let bd = wallet_parser.parser.get_birthday();
        let wallet_version = wallet_parser.parser.get_wallet_version();

        if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
            self.log_buffer
                .lock()
                .unwrap()
                .push(format!("Error installing crypto provider: {:?}", e));
        }

        let zc = load_clientconfig(
            Uri::from_static("https://na.zec.rocks:443"),
            None,
            ChainType::Mainnet,
            WalletSettings {
                sync_config: SyncConfig {
                    transparent_address_discovery: TransparentAddressDiscovery::recovery(),
                },
            },
            NonZero::new(1).unwrap(),
        )
        .unwrap();

        let initial_bh: u32 = bd.try_into().unwrap();
        let lw = LightWallet::new(
            ChainType::Mainnet,
            WalletBase::Mnemonic {
                mnemonic: Mnemonic::from_entropy(seed).unwrap(),
                no_of_accounts: NonZero::new(1).unwrap(),
            },
            initial_bh.into(),
            WalletSettings {
                sync_config: SyncConfig {
                    transparent_address_discovery: TransparentAddressDiscovery::recovery(),
                },
            },
        )
        .unwrap();

        let mut light_client = lightclient::LightClient::create_from_wallet(lw, zc, true).unwrap();

        let mnemonic = {
            let wallet_guard = light_client.wallet.lock().await;
            let mnemonic = wallet_guard.mnemonic().cloned();
            mnemonic
        };

        self.log_buffer
            .lock()
            .unwrap()
            .push(format!("=== WALLET VERSION: {} ===", wallet_version));

        self.log_buffer
            .lock()
            .unwrap()
            .push(format!("Mnemonic: {}", mnemonic.unwrap().to_string()));
        self.log_buffer
            .lock()
            .unwrap()
            .push(format!("Starting sync from birthday: {}", bd));
        match light_client.sync().await {
            Ok(_) => self
                .log_buffer
                .lock()
                .unwrap()
                .push("Sync started".to_string()),
            Err(e) => self
                .log_buffer
                .lock()
                .unwrap()
                .push(format!("Error starting syncing: {}", e)),
        }

        let mut interval = tokio::time::interval(Duration::from_secs(1));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            interval.tick().await;
            match light_client.poll_sync() {
                PollReport::NoHandle => {}
                PollReport::NotReady => {
                    let wallet_guard = light_client.wallet.lock().await;
                    match sync_status(&*wallet_guard).await {
                        Ok(status) => {
                            *self.progress.lock().unwrap() =
                                status.percentage_total_outputs_scanned;
                            self.log_buffer.lock().unwrap().push(format!("{}", status));
                        }
                        Err(e) => {
                            self.log_buffer.lock().unwrap().push(format!("{}", e));
                            continue;
                        }
                    };
                }
                PollReport::Ready(result) => match result {
                    Ok(sync_result) => {
                        self.log_buffer
                            .lock()
                            .unwrap()
                            .push(format!("Sync result: {:?}", sync_result));
                        let balances = light_client
                            .wallet
                            .lock()
                            .await
                            .account_balance(zip32::AccountId::try_from(0).unwrap())
                            .await
                            .unwrap();
                        let final_balance = balances.total_transparent_balance.unwrap()
                            + balances.total_sapling_balance.unwrap()
                            + balances.total_orchard_balance.unwrap();
                        let balance_in_zec =
                            final_balance.unwrap() / NonZero::new(10u64.pow(8)).unwrap();
                        self.log_buffer
                            .lock()
                            .unwrap()
                            .push(format!("Total ZEC found: {}", balance_in_zec.into_u64()));

                        *self.sync_complete.lock().unwrap() = true;

                        break;
                    }
                    Err(_e) => {
                        self.log_buffer
                            .lock()
                            .unwrap()
                            .push("Error. Resuming sync".to_string());
                        self.log_buffer
                            .lock()
                            .unwrap()
                            .push("Restarting sync".to_string());
                        match light_client.sync().await {
                            Ok(_) => self
                                .log_buffer
                                .lock()
                                .unwrap()
                                .push("Sync resumed".to_string()),
                            Err(e) => self.log_buffer.lock().unwrap().push(format!("{}", e)),
                        }
                        continue;
                    }
                },
            };
        }

        match light_client.await_sync().await {
            Ok(_) => {
                self.log_buffer
                    .lock()
                    .unwrap()
                    .push("Sync finished".to_string());
            }
            Err(e) => self.log_buffer.lock().unwrap().push(format!("{}", e)),
        }

        light_client
    }

    pub async fn start_wallet_sync_from_mnemonic(
        &self,
        mnemonic_str: String,
        birthday: Option<u32>,
    ) -> LightClient {
        const INITIAL_WINDOW: u32 = 10;
        const GAP_LIMIT: u32 = 5;

        if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
            self.log_buffer
                .lock()
                .unwrap()
                .push(format!("Error installing crypto provider: {:?}", e));
        }

        let birthday = birthday.unwrap_or_default();
        let wallet_settings = WalletSettings {
            sync_config: SyncConfig {
                transparent_address_discovery: TransparentAddressDiscovery::recovery(),
            },
        };
        let mut no_of_accounts = INITIAL_WINDOW;
        let mut light_client =
            self.create_mnemonic_client(&mnemonic_str, birthday, no_of_accounts, &wallet_settings);

        // Sync using the current account window; expand and retry if needed
        loop {
            self.log_buffer.lock().unwrap().push(format!(
                "Scanning {} account(s) from birthday {}",
                no_of_accounts, birthday
            ));

            // Start current wallet sync attempt
            match light_client.sync().await {
                Ok(_) => {}
                Err(e) => self
                    .log_buffer
                    .lock()
                    .unwrap()
                    .push(format!("Error starting sync: {}", e)),
            }

            let mut interval = tokio::time::interval(Duration::from_secs(1));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            // Poll current wallet sync until completion
            loop {
                interval.tick().await;
                match light_client.poll_sync() {
                    PollReport::NoHandle => (),
                    PollReport::NotReady => {
                        let wallet = light_client.wallet.lock().await;
                        match sync_status(&*wallet).await {
                            Ok(status) => {
                                self.log_buffer.lock().unwrap().push(format!("{}", status));
                                *self.progress.lock().unwrap() =
                                    status.percentage_total_outputs_scanned;
                            }
                            Err(e) => {
                                self.log_buffer.lock().unwrap().push(format!("{}", e));
                                continue;
                            }
                        };
                    }
                    PollReport::Ready(result) => match result {
                        Ok(sync_result) => {
                            self.log_buffer
                                .lock()
                                .unwrap()
                                .push(format!("Sync result: {:?}", sync_result));
                            break;
                        }
                        Err(_e) => {
                            self.log_buffer
                                .lock()
                                .unwrap()
                                .push("Error. Resuming sync".to_string());
                            match light_client.sync().await {
                                Ok(_) => self
                                    .log_buffer
                                    .lock()
                                    .unwrap()
                                    .push("Sync resumed".to_string()),
                                Err(e) => self.log_buffer.lock().unwrap().push(format!("{}", e)),
                            }
                            continue;
                        }
                    },
                }
            }

            // Check highest account with any shielded note history (including spent)
            let wallet_guard = light_client.wallet.lock().await;
            let sapling_max: Option<u32> = wallet_guard
                .wallet_transactions
                .values()
                .flat_map(|tx| tx.sapling_notes())
                .map(|note| u32::from(note.key_id().account_id))
                .max();
            let orchard_max: Option<u32> = wallet_guard
                .wallet_transactions
                .values()
                .flat_map(|tx| tx.orchard_notes())
                .map(|note| u32::from(note.key_id().account_id))
                .max();
            drop(wallet_guard);
            let highest_active = sapling_max.max(orchard_max);

            let needs_expansion =
                needs_account_expansion(highest_active, no_of_accounts, GAP_LIMIT);

            if !needs_expansion {
                break;
            }

            // Recreate wallet with a larger account window
            no_of_accounts += GAP_LIMIT;
            self.log_buffer.lock().unwrap().push(format!(
                "Activity near bound (highest: {}), expanding to {} accounts",
                highest_active.unwrap(),
                no_of_accounts
            ));
            light_client = self.create_mnemonic_client(
                &mnemonic_str,
                birthday,
                no_of_accounts,
                &wallet_settings,
            );
        }

        // Report total balance across all accounts
        let wallet_guard = light_client.wallet.lock().await;
        let mut total_zat: u64 = 0;
        for i in 0..no_of_accounts {
            let account_id = zip32::AccountId::try_from(i).unwrap();
            if let Ok(b) = wallet_guard.account_balance(account_id).await {
                let pool_sum = b.total_transparent_balance.unwrap()
                    + b.total_sapling_balance.unwrap()
                    + b.total_orchard_balance.unwrap();
                if let Some(zat) = pool_sum {
                    total_zat += u64::from(zat);
                }
            }
        }
        drop(wallet_guard);
        self.log_buffer
            .lock()
            .unwrap()
            .push(format!("Total ZEC found: {}", total_zat / 10u64.pow(8)));
        *self.sync_complete.lock().unwrap() = true;

        light_client
    }

    fn create_mnemonic_client(
        &self,
        mnemonic_str: &str,
        birthday: u32,
        no_of_accounts: u32,
        wallet_settings: &WalletSettings,
    ) -> LightClient {
        let zc = load_clientconfig(
            Uri::from_static(DEFAULT_LIGHTWALLETD_SERVER),
            None,
            ChainType::Mainnet,
            wallet_settings.clone(),
            NonZero::new(no_of_accounts).unwrap(),
        )
        .unwrap();
        let lw = LightWallet::new(
            ChainType::Mainnet,
            WalletBase::Mnemonic {
                mnemonic: Mnemonic::<English>::from_str(mnemonic_str).unwrap(),
                no_of_accounts: NonZero::new(no_of_accounts).unwrap(),
            },
            birthday.into(),
            wallet_settings.clone(),
        )
        .unwrap();
        LightClient::create_from_wallet(lw, zc, true).unwrap()
    }
}

impl Mountable for SyncView {
    fn mount(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()> {
        assert!(app
            .mount(
                Id::ProgressBar,
                Box::new(SyncBar::default()),
                Vec::default()
            )
            .is_ok());
        Ok(())
    }
}

impl Renderable for SyncView {
    fn render(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(3), Constraint::Percentage(80)])
            .split(f.area());
        app.view(&Id::ProgressBar, f, chunks[0]);
        app.view(&Id::SyncLog, f, chunks[1]);
    }
}

fn needs_account_expansion(highest_active: Option<u32>, window: u32, gap_limit: u32) -> bool {
    match highest_active {
        Some(h) => h >= window.saturating_sub(gap_limit),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[derive(Debug, serde::Deserialize, Clone)]
    pub struct WalletTestVector {
        pub mnemonic: String,
        pub birthday: Option<u64>,
    }

    #[tokio::test]
    async fn test_mnemonic_vectors_from_file() {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("testvectors/mnemonic.json");

        println!("Using test vector file: {}", path.display());

        let json = fs::read_to_string(&path).expect("Failed to read test vector file");
        let vectors: Vec<WalletTestVector> =
            serde_json::from_str(&json).expect("Failed to parse test vectors");

        for (i, vec) in vectors.iter().enumerate() {
            println!("\n Running vector {i}: {}", &vec.mnemonic);
            println!("from birthday: {}", vec.birthday.unwrap_or(0));

            let log_buffer = Arc::new(Mutex::new(vec![]));
            let view = SyncView::new_with_log(log_buffer.clone());

            let client = view
                .start_wallet_sync_from_mnemonic(
                    vec.mnemonic.clone(),
                    Some(vec.birthday.unwrap_or(0) as u32),
                )
                .await;

            let complete = *view.sync_complete.lock().unwrap();
            assert!(
                complete,
                "Vector {i} failed: Sync did not complete\nMnemonic: {}\nBirthday: {:?}",
                vec.mnemonic, vec.birthday
            );

            let wallet_guard = client.wallet.lock().await;
            let mut total_zat: u64 = 0;
            for acct in 0..5u32 {
                let account_id = zip32::AccountId::try_from(acct).unwrap();
                if let Ok(b) = wallet_guard.account_balance(account_id).await {
                    let pool_sum = b.total_transparent_balance.unwrap()
                        + b.total_sapling_balance.unwrap()
                        + b.total_orchard_balance.unwrap();
                    if let Some(zat) = pool_sum {
                        total_zat += u64::from(zat);
                    }
                }
            }
            drop(wallet_guard);

            println!(
                "Vector {i} passed! Found balance: {} ZEC",
                total_zat / 10u64.pow(8)
            );
        }
    }

    mod expansion_predicate {
        use super::super::needs_account_expansion;
        #[test]
        fn no_activity_does_not_expand() {
            assert!(!needs_account_expansion(None, 10, 5));
        }

        #[test]
        fn activity_below_threshold_does_not_expand() {
            assert!(!needs_account_expansion(Some(4), 10, 5));
        }

        #[test]
        fn activity_at_threshold_expands() {
            assert!(needs_account_expansion(Some(5), 10, 5));
        }

        #[test]
        fn saturating_sub_edge_case_expands() {
            assert!(needs_account_expansion(Some(0), 3, 5));
        }
    }

    mod discovery_loop {
        use super::super::needs_account_expansion;

        /// Test-only helper that simulates repeated account-window expansion
        /// decisions from canned per-round `highest_active` values.
        fn simulate_discovery(initial_window: u32, gap_limit: u32, rounds: &[Option<u32>]) -> u32 {
            let mut window = initial_window;
            for highest_active in rounds {
                if !needs_account_expansion(*highest_active, window, gap_limit) {
                    break;
                }
                window += gap_limit;
            }
            window
        }

        #[test]
        fn no_activity_no_expansion() {
            assert_eq!(simulate_discovery(10, 5, &[None]), 10);
        }

        #[test]
        fn one_expansion_then_settles() {
            // Round 0: window=10, highest=8, threshold=5 → 8 >= 5 → expand to 15
            // Round 1: window=15, highest=8, threshold=10 → 8 < 10 → stop
            assert_eq!(simulate_discovery(10, 5, &[Some(8), Some(8)]), 15);
        }

        #[test]
        fn two_expansions_then_settles() {
            // Round 0: window=10, highest=7, threshold=5 → 7 >= 5 → expand to 15
            // Round 1: window=15, highest=12, threshold=10 → 12 >= 10 → expand to 20
            // Round 2: window=20, highest=12, threshold=15 → 12 < 15 → stop
            assert_eq!(
                simulate_discovery(10, 5, &[Some(7), Some(12), Some(12)]),
                20
            );
        }
    }
}
