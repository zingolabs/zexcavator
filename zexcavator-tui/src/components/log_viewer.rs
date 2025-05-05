use bip0039::{English, Mnemonic};
use pepper_sync::sync_status;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use zexcavator_lib::parser::WalletParserFactory;

use std::time::Duration;

pub type LogBuffer = Arc<Mutex<Vec<String>>>;

pub fn new_log_buffer() -> LogBuffer {
    Arc::new(Mutex::new(Vec::new()))
}

pub fn start_wallet_sync(logs: LogBuffer, path: PathBuf) {
    std::thread::spawn(move || {
        let wallet_parser = WalletParserFactory::read(path.to_str().unwrap()).unwrap();

        let seed = wallet_parser.parser.get_wallet_seed();
        let bd = wallet_parser.parser.get_birthday();

        if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
            logs.lock()
                .unwrap()
                .push(format!("Error installing crypto provider: {:?}", e));
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
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
                logs.lock()
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

        rt.block_on(async {
            logs.lock()
                .unwrap()
                .push(format!("Starting sync from birthday: {}", bd));
            match lc.sync().await {
                Ok(_) => logs.lock().unwrap().push("Sync started".to_string()),
                Err(e) => logs
                    .lock()
                    .unwrap()
                    .push(format!("Error starting syncing: {}", e)),
            }

            loop {
                sleep(Duration::from_secs(1)).await;
                match lc.poll_sync() {
                    PollReport::NoHandle => {
                        logs.lock().unwrap().push("No handle".to_string());
                    }
                    PollReport::NotReady => {
                        let wallet_guard = lc.wallet.lock().await;
                        match sync_status(&*wallet_guard).await {
                            Ok(status) => {
                                logs.lock().unwrap().push(format!("{}", status));
                            }
                            Err(e) => {
                                logs.lock().unwrap().push(format!("{}", e));
                                continue;
                            }
                        };
                    }
                    PollReport::Ready(result) => match result {
                        Ok(sync_result) => {
                            logs.lock()
                                .unwrap()
                                .push(format!("Sync result: {:?}", sync_result));
                            let balances = lc.do_balance().await;
                            logs.lock()
                                .unwrap()
                                .push(format!("Balances: {:?}", balances));
                            break;
                        }
                        Err(e) => {
                            logs.lock().unwrap().push(format!("{}", e));
                            logs.lock().unwrap().push("Restarting sync".to_string());
                            match lc.sync().await {
                                Ok(_) => logs.lock().unwrap().push("Sync resumed".to_string()),
                                Err(e) => logs.lock().unwrap().push(format!("{}", e)),
                            }
                            continue;
                        }
                    },
                };
            }

            match lc.await_sync().await {
                Ok(_) => logs.lock().unwrap().push("Sync finished".to_string()),
                Err(e) => logs.lock().unwrap().push(format!("{}", e)),
            }
        });
    });
}

pub fn start_wallet_sync_from_mnemonic(logs: LogBuffer, mnemonic_str: String, birthday: u32) {
    std::thread::spawn(move || {
        if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
            logs.lock()
                .unwrap()
                .push(format!("Error installing crypto provider: {:?}", e));
        }

        let rt = tokio::runtime::Runtime::new().unwrap();
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
                logs.lock()
                    .unwrap()
                    .push(format!("Error loading client config: {}", e));
                return;
            }
        };

        let mnemonic = Mnemonic::<English>::from_str(&mnemonic_str).unwrap();

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

        rt.block_on(async {
            logs.lock()
                .unwrap()
                .push(format!("Starting sync from birthday: {}", birthday));
            match lc.sync().await {
                Ok(_) => logs.lock().unwrap().push("Sync started".to_string()),
                Err(e) => logs
                    .lock()
                    .unwrap()
                    .push(format!("Error starting syncing: {}", e)),
            }

            loop {
                sleep(Duration::from_secs(1)).await;
                match lc.poll_sync() {
                    PollReport::NoHandle => {
                        logs.lock().unwrap().push("No handle".to_string());
                    }
                    PollReport::NotReady => {
                        let wallet_guard = lc.wallet.lock().await;
                        match sync_status(&*wallet_guard).await {
                            Ok(status) => {
                                logs.lock().unwrap().push(format!("{}", status));
                            }
                            Err(e) => {
                                logs.lock().unwrap().push(format!("{}", e));
                                continue;
                            }
                        };
                    }
                    PollReport::Ready(result) => match result {
                        Ok(sync_result) => {
                            logs.lock()
                                .unwrap()
                                .push(format!("Sync result: {:?}", sync_result));
                            let balances = lc.do_balance().await;
                            logs.lock()
                                .unwrap()
                                .push(format!("Balances: {:?}", balances));
                            break;
                        }
                        Err(e) => {
                            logs.lock().unwrap().push(format!("{}", e));
                            logs.lock().unwrap().push("Restarting sync".to_string());
                            match lc.sync().await {
                                Ok(_) => logs.lock().unwrap().push("Sync resumed".to_string()),
                                Err(e) => logs.lock().unwrap().push(format!("{}", e)),
                            }
                            continue;
                        }
                    },
                };
            }

            match lc.await_sync().await {
                Ok(_) => logs.lock().unwrap().push("Sync finished".to_string()),
                Err(e) => logs.lock().unwrap().push(format!("{}", e)),
            }
        });
    });
}

pub struct LogViewer {
    logs: LogBuffer,
}

impl LogViewer {
    pub fn new(logs: LogBuffer) -> Self {
        Self { logs }
    }
}

impl Default for LogViewer {
    fn default() -> Self {
        Self::new(new_log_buffer())
    }
}

use http::Uri;
use pepper_sync::sync::{SyncConfig, TransparentAddressDiscovery};
use tokio::time::sleep;
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::widgets::Wrap;
use tuirealm::{Component, Event, Frame, MockComponent, NoUserEvent};
use zingolib::config::{ChainType, load_clientconfig};
use zingolib::data::PollReport;
use zingolib::lightclient;
use zingolib::wallet::{LightWallet, WalletBase, WalletSettings};

use crate::Msg;

impl MockComponent for LogViewer {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        use tuirealm::ratatui::text::{Line, Span, Text};
        use tuirealm::ratatui::widgets::{Block, Borders, Paragraph};

        let log_lines = self.logs.lock().unwrap();
        let text = Text::from(
            log_lines
                .iter()
                .map(|l| Line::from(Span::raw(l)))
                .collect::<Vec<_>>(),
        );

        let scroll_offset = log_lines.len().saturating_sub(area.height as usize - 2);

        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Sync Log").borders(Borders::ALL))
            .scroll((scroll_offset as u16, 0))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        todo!()
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        todo!()
    }

    fn state(&self) -> tuirealm::State {
        todo!()
    }

    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        todo!()
    }

    // Other trait methods (attr, state, etc.) as no-ops
}

impl Component<Msg, NoUserEvent> for LogViewer {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None // Tick drives redraw
    }
}
