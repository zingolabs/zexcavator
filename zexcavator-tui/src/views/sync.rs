use std::path::PathBuf;
use std::str::FromStr;
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
use zingolib::lightclient::{self};
use zingolib::wallet::{LightWallet, WalletBase, WalletSettings};

use crate::components::log_viewer::LogBuffer;
use crate::{Id, Msg};

use super::{Mountable, Renderable};

#[derive(Debug, Clone)]
pub struct SyncView {
    log_buffer: LogBuffer,
}

impl SyncView {
    pub fn new_with_log(log_buffer: LogBuffer) -> Self {
        Self { log_buffer }
    }

    pub async fn start_wallet_sync_from_path(&self, path: PathBuf) {
        let wallet_parser = WalletParserFactory::read(path.to_str().unwrap()).unwrap();

        let seed = wallet_parser.parser.get_wallet_seed();
        let bd = wallet_parser.parser.get_birthday();

        if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
            self.log_buffer
                .lock()
                .unwrap()
                .push(format!("Error installing crypto provider: {:?}", e));
        }

        let zc = match load_clientconfig(
            Uri::from_static("https://na.zec.rocks:443"),
            None,
            ChainType::Mainnet,
            WalletSettings {
                sync_config: SyncConfig {
                    transparent_address_discovery: TransparentAddressDiscovery::recovery(),
                },
            },
        ) {
            Ok(zc) => zc,
            Err(e) => {
                self.log_buffer
                    .lock()
                    .unwrap()
                    .push(format!("Error loading client config: {}", e));
                return;
            }
        };

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

        let mut lc = lightclient::LightClient::create_from_wallet(lw, zc, true).unwrap();

        self.log_buffer
            .lock()
            .unwrap()
            .push(format!("Starting sync from birthday: {}", bd));
        match lc.sync().await {
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
            match lc.poll_sync() {
                PollReport::NoHandle => {
                    self.log_buffer
                        .lock()
                        .unwrap()
                        .push("No handle".to_string());
                }
                PollReport::NotReady => {
                    let wallet_guard = lc.wallet.lock().await;
                    match sync_status(&*wallet_guard).await {
                        Ok(status) => {
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
                        let balances = lc.do_balance().await;
                        self.log_buffer
                            .lock()
                            .unwrap()
                            .push(format!("Balances: {:?}", balances));
                        break;
                    }
                    Err(e) => {
                        self.log_buffer.lock().unwrap().push(format!("{}", e));
                        self.log_buffer
                            .lock()
                            .unwrap()
                            .push("Restarting sync".to_string());
                        match lc.sync().await {
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

        match lc.await_sync().await {
            Ok(_) => self
                .log_buffer
                .lock()
                .unwrap()
                .push("Sync finished".to_string()),
            Err(e) => self.log_buffer.lock().unwrap().push(format!("{}", e)),
        }
    }

    pub async fn start_wallet_sync_from_mnemonic(
        &self,
        mnemonic_str: String,
        birthday: Option<u32>,
    ) {
        if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
            self.log_buffer
                .lock()
                .unwrap()
                .push(format!("Error installing crypto provider: {:?}", e));
        }

        let zc = match load_clientconfig(
            Uri::from_static("https://na.zec.rocks:443"),
            None,
            ChainType::Mainnet,
            WalletSettings {
                sync_config: SyncConfig {
                    transparent_address_discovery: TransparentAddressDiscovery::recovery(),
                },
            },
        ) {
            Ok(zc) => zc,
            Err(e) => {
                self.log_buffer
                    .lock()
                    .unwrap()
                    .push(format!("Error loading client config: {}", e));
                return;
            }
        };

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

        let mut lc = lightclient::LightClient::create_from_wallet(lw, zc, true).unwrap();

        self.log_buffer
            .lock()
            .unwrap()
            .push(format!("Starting sync from birthday: {}", birthday));
        match lc.sync().await {
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
            match lc.poll_sync() {
                PollReport::NoHandle => {
                    self.log_buffer
                        .lock()
                        .unwrap()
                        .push("No handle".to_string());
                }
                PollReport::NotReady => {
                    let wallet_guard = lc.wallet.lock().await;
                    match sync_status(&*wallet_guard).await {
                        Ok(status) => {
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
                        let balances = lc.do_balance().await;
                        self.log_buffer
                            .lock()
                            .unwrap()
                            .push(format!("Balances: {:?}", balances));
                        break;
                    }
                    Err(e) => {
                        self.log_buffer.lock().unwrap().push(format!("{}", e));
                        self.log_buffer
                            .lock()
                            .unwrap()
                            .push("Restarting sync".to_string());
                        match lc.sync().await {
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

        match lc.await_sync().await {
            Ok(_) => self
                .log_buffer
                .lock()
                .unwrap()
                .push("Sync finished".to_string()),
            Err(e) => self.log_buffer.lock().unwrap().push(format!("{}", e)),
        }
    }
}

impl Mountable for SyncView {
    fn mount(_app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()> {
        Ok(())
    }
}

impl Renderable for SyncView {
    fn render(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(100)])
            .split(f.area());
        app.view(&Id::SyncLog, f, chunks[0]);
    }
}
