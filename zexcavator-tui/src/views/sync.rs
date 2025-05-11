use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bip0039::{English, Mnemonic};
use http::Uri;
use pepper_sync::sync::{SyncConfig, TransparentAddressDiscovery};
use pepper_sync::sync_status;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::{Application, Frame, NoUserEvent};
use zexcavator_lib::parser::WalletParserFactory;
use zingolib::config::{ChainType, load_clientconfig};
use zingolib::data::PollReport;
use zingolib::lightclient::{self, LightClient};
use zingolib::wallet::{LightWallet, WalletBase, WalletSettings};

use crate::components::log_viewer::LogBuffer;
use crate::components::sync_bar::SyncBar;
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
        )
        .unwrap();

        let initial_bh: u32 = bd.try_into().unwrap();
        let lw = LightWallet::new(
            ChainType::Mainnet,
            WalletBase::SeedBytes(seed),
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
            .push(format!("Mnemonic: {}", mnemonic.unwrap().0));
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
                        let balances = light_client.do_balance().await;
                        let final_balance = balances.confirmed_transparent_balance.unwrap()
                            + balances.verified_sapling_balance.unwrap()
                            + balances.verified_orchard_balance.unwrap();
                        let balance_in_zec = final_balance / 10u64.pow(8);
                        self.log_buffer
                            .lock()
                            .unwrap()
                            .push(format!("Total ZEC found: {}", balance_in_zec));

                        *self.sync_complete.lock().unwrap() = true;

                        break;
                    }
                    Err(e) => {
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
        )
        .unwrap();

        let mnemonic = Mnemonic::<English>::from_str(&mnemonic_str).unwrap();

        let birthday = birthday.unwrap_or_default();

        let lw = LightWallet::new(
            ChainType::Mainnet,
            WalletBase::Mnemonic(mnemonic),
            birthday.into(),
            WalletSettings {
                sync_config: SyncConfig {
                    transparent_address_discovery: TransparentAddressDiscovery::recovery(),
                },
            },
        )
        .unwrap();

        let mut light_client = LightClient::create_from_wallet(lw, zc, true).unwrap();

        self.log_buffer
            .lock()
            .unwrap()
            .push(format!("Starting sync from birthday: {}", birthday));
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
                        let balances = light_client.do_balance().await;

                        let final_balance = balances.confirmed_transparent_balance.unwrap()
                            + balances.verified_sapling_balance.unwrap()
                            + balances.verified_orchard_balance.unwrap();
                        let balance_in_zec = final_balance / 10u64.pow(8);
                        self.log_buffer
                            .lock()
                            .unwrap()
                            .push(format!("Total ZEC found: {}", balance_in_zec));
                        *self.sync_complete.lock().unwrap() = true;

                        break;
                    }
                    Err(e) => {
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
            }
        }

        match light_client.sync().await {
            Ok(_) => {
                self.log_buffer
                    .lock()
                    .unwrap()
                    .push("Sync finished".to_string());
            }
            Err(e) => {
                self.log_buffer.lock().unwrap().push(format!("{}", e));
            }
        }

        light_client
    }
}

impl Mountable for SyncView {
    fn mount(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()> {
        assert!(
            app.mount(
                Id::ProgressBar,
                Box::new(SyncBar::default()),
                Vec::default()
            )
            .is_ok()
        );
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
