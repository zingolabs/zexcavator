//! ## Model
//!
//! app model

use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::RwLock;
use tuirealm::event::NoUserEvent;
use tuirealm::props::{PropPayload, PropValue};
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge};
use tuirealm::{Application, AttrValue, Attribute, EventListenerCfg, Update};
use zingolib::lightclient::LightClient;

use crate::components::HandleMessage;
use crate::components::log_viewer::{LogViewer, SyncSource, new_log_buffer};
use crate::components::menu::MenuOptions;
use crate::views::export::send::ExportSendView;
use crate::views::export::zewif::ExportZewifView;
use crate::views::export::zingolib::ExportZingolibView;
use crate::views::export::{ExportOptions, ExportView};
use crate::views::main_menu::MainMenu;
use crate::views::sync::SyncView;
use crate::views::zecwallet::ZecwalletMenu;
use crate::views::zecwallet::from_mnemonic::ZecwalletFromMnemonic;
use crate::views::zecwallet::from_path::ZecwalletFromPath;
use crate::views::{Mountable, Renderable, main_menu};

use super::{Id, Msg};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    MainMenu,
    Syncing,
    ZecwalletInput,
    ZecwalletFromPath,
    ZecwalletFromMnemonic,
    ZcashdInput,
    Result,
    ExportZewif,
    ExportSend,
    ExportZingolib,
}

pub struct Model<T>
where
    T: TerminalAdapter,
{
    /// Application
    pub app: Application<Id, Msg, NoUserEvent>,
    /// Indicates that the application must quit
    pub quit: bool,
    /// Tells whether to redraw interface
    pub redraw: bool,
    /// Used to draw to terminal
    pub terminal: TerminalBridge<T>,
    /// Active screen
    pub screen: Screen,
    pub sync_view: Arc<SyncView>,
    pub light_client: Arc<RwLock<Option<LightClient>>>,
    pub export_menu: ExportView,
    pub export_zewif: ExportZewifView,
    pub export_send: ExportSendView,
    pub export_zingolib: ExportZingolibView,
}

impl Default for Model<CrosstermTerminalAdapter> {
    fn default() -> Self {
        let log_buffer_path = new_log_buffer();
        let light_client = Arc::new(RwLock::new(None));
        let export_menu = ExportView::new(Arc::clone(&light_client));
        let export_zewif = ExportZewifView::new(Arc::clone(&light_client));
        let export_send = ExportSendView::new(Arc::clone(&light_client));
        let export_zingolib = ExportZingolibView::new(Arc::clone(&light_client));

        let mut app = Self::init_app(
            Arc::clone(&light_client),
            export_menu.clone(),
            export_zewif.clone(),
            export_send.clone(),
            export_zingolib.clone(),
        );

        assert!(
            app.mount(
                Id::SyncLog,
                Box::new(LogViewer::new(log_buffer_path.clone())),
                Vec::default()
            )
            .is_ok()
        );

        Self {
            app,
            quit: false,
            redraw: true,
            screen: Screen::MainMenu,
            terminal: TerminalBridge::init_crossterm().expect("Cannot initialize terminal"),
            sync_view: Arc::new(SyncView::new_with_log(log_buffer_path)),
            light_client,
            export_menu,
            export_zewif,
            export_send,
            export_zingolib,
        }
    }
}

impl<T> Model<T>
where
    T: TerminalAdapter,
{
    pub fn view(&mut self) {
        assert!(
            self.terminal
                .draw(|f| {
                    match self.screen {
                        Screen::MainMenu => main_menu::render(&mut self.app, f),
                        Screen::Syncing => SyncView::render(&mut self.app, f),
                        Screen::ZecwalletInput => ZecwalletMenu::render(&mut self.app, f),
                        Screen::ZecwalletFromPath => ZecwalletFromPath::render(&mut self.app, f),
                        Screen::ZecwalletFromMnemonic => {
                            ZecwalletFromMnemonic::render(&mut self.app, f)
                        }
                        Screen::ZcashdInput => todo!(),
                        Screen::Result => {
                            let area = f.area();
                            self.app.view(&Id::ExportView, f, area);
                        }
                        Screen::ExportSend => {
                            let area = f.area();
                            self.app.view(&Id::ExportSend, f, area);
                        }
                        Screen::ExportZewif => {
                            let area = f.area();
                            self.app.view(&Id::ExportZewif, f, area);
                        }
                        Screen::ExportZingolib => {
                            let area = f.area();
                            self.app.view(&Id::ExportZingolib, f, area);
                        }
                    }
                })
                .is_ok()
        );
    }

    fn init_app(
        lc: Arc<RwLock<Option<LightClient>>>,
        export_menu: ExportView,
        export_zewif: ExportZewifView,
        export_send: ExportSendView,
        export_zingolib: ExportZingolibView,
    ) -> Application<Id, Msg, NoUserEvent> {
        // Setup application
        // NOTE: NoUserEvent is a shorthand to tell tui-realm we're not going to use any custom user event
        // NOTE: the event listener is configured to use the default crossterm input listener and to raise a Tick event each second
        // which we will use to update the clock

        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default()
                .crossterm_input_listener(Duration::from_millis(20), 3)
                .poll_timeout(Duration::from_millis(10))
                .tick_interval(Duration::from_secs(1)),
        );

        // Mount components:

        // Mount main menu
        assert!(MainMenu::mount(&mut app).is_ok());

        // Mount Zecwallet view
        assert!(ZecwalletMenu::mount(&mut app).is_ok());

        // let log_buffer = new_log_buffer();

        assert!(ZecwalletFromPath::mount(&mut app).is_ok());

        assert!(ZecwalletFromMnemonic::mount(&mut app).is_ok());

        assert!(SyncView::mount(&mut app).is_ok());
        assert!(
            app.mount(Id::ExportView, Box::new(export_menu), Vec::default())
                .is_ok()
        );

        // Mount export zewif view
        assert!(
            app.mount(Id::ExportZewif, Box::new(export_zewif), Vec::default())
                .is_ok()
        );

        // Mount export send view
        assert!(
            app.mount(Id::ExportSend, Box::new(export_send), Vec::default())
                .is_ok()
        );

        // Mount export zingolib view
        assert!(
            app.mount(
                Id::ExportZingolib,
                Box::new(export_zingolib),
                Vec::default()
            )
            .is_ok()
        );

        // Focus main menu
        assert!(app.active(&Id::MainMenu).is_ok());

        app
    }
}

// Update loop
impl<T> Update<Msg> for Model<T>
where
    T: TerminalAdapter,
{
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if *self.sync_view.sync_complete.lock().unwrap() {
            let export_menu = self.export_menu.clone();
            tokio::spawn(async move {
                let balance = export_menu.load_balance().await;
                if let Some(b) = balance {
                    let mut guard = export_menu.balance.write().await;
                    guard.replace(b);
                    drop(guard);
                }
            });

            self.navigate_to(Screen::Result);
            *self.sync_view.sync_complete.lock().unwrap() = false;
            self.redraw = true;
        }
        if self.screen == Screen::Syncing {
            let balance_loaded = {
                if let Ok(balance_guard) = self.export_menu.balance.try_read() {
                    balance_guard.is_some()
                } else {
                    false
                }
            };
            if balance_loaded {
                self.navigate_to(Screen::Result);
            }
            self.redraw = true;
        }
        if self.screen == Screen::Syncing {
            let progress = *self.sync_view.get_progress().lock().unwrap();
            let _ = self.app.attr(
                &Id::ProgressBar,
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::F32(progress))),
            );
            self.redraw = true;
        }

        if let Some(msg) = msg {
            // Set redraw
            self.redraw = true;
            // Match message
            match msg {
                Msg::AppClose => {
                    self.quit = true; // Terminate
                    None
                }
                Msg::SeedInputChanged(s) => {
                    assert!(
                        self.app
                            .attr(
                                &Id::ZecwalletFromPath,
                                Attribute::Text,
                                AttrValue::String(s)
                            )
                            .is_ok()
                    );
                    None
                }
                Msg::None => None,
                Msg::SeedInputValidate(path) => {
                    match ZecwalletFromPath::validate_path(PathBuf::from_str(&path).unwrap()) {
                        Err(_) => None::<Msg>,
                        Ok(_) => {
                            return None;
                        }
                    };
                    None
                }
                Msg::Start => {
                    self.screen = Screen::MainMenu;
                    self.app.active(&Id::MainMenu);
                    None
                }
                Msg::MenuCursorMove(_) => None,
                Msg::MenuSelected(label) => {
                    if self.screen == Screen::Result {
                        match ExportOptions::from_label(&label).unwrap() {
                            ExportOptions::Zingolib => {
                                self.navigate_to(Screen::ExportZingolib);
                                let view = self.export_zingolib.clone();

                                tokio::spawn(async move {
                                    let path_or_err = view
                                        .do_save()
                                        .await
                                        .map_err(|e| format!("Error: {}", e))
                                        .map(|p| p.to_string());
                                    *view.saved_path.lock().unwrap() =
                                        Some(path_or_err.unwrap_or_else(|e| e));
                                });
                                return None;
                            }
                            ExportOptions::ZeWIF => {
                                self.navigate_to(Screen::ExportZewif);
                                let view = self.export_zewif.clone();
                                tokio::spawn(async move {
                                    let path_or_err = view
                                        .do_save()
                                        .await
                                        .map_err(|e| format!("Error: {}", e))
                                        .map(|p| p.to_string());
                                    *view.saved_path.lock().unwrap() =
                                        Some(path_or_err.unwrap_or_else(|e| e));
                                });
                                return None;
                            }
                            ExportOptions::Send => {
                                todo!();
                            }
                            ExportOptions::Cancel => {
                                self.navigate_to(Screen::MainMenu);
                            }
                        }
                    }

                    let msg = Msg::MenuSelected(label.clone());
                    match self.screen {
                        Screen::MainMenu => MainMenu::handle_message(msg, self),
                        Screen::ZecwalletInput
                        | Screen::ZecwalletFromPath
                        | Screen::ZecwalletFromMnemonic => ZecwalletMenu::handle_message(msg, self),
                        Screen::Result => ExportView::handle_message(msg, self),
                        _ => None,
                    }
                }
                Msg::MnemonicInputChanged(s) => {
                    assert!(
                        self.app
                            .attr(&Id::MnemonicInput, Attribute::Text, AttrValue::String(s))
                            .is_ok()
                    );
                    None
                }
                Msg::MnemonicInputBlur => {
                    assert!(self.app.active(&Id::BirthdayInput).is_ok());
                    None
                }
                Msg::FromPathInputBlur => {
                    assert!(self.app.active(&Id::ZecwalletFromPathButton).is_ok());
                    None
                }
                Msg::BirthdayInputChanged(birthday) => {
                    assert!(
                        self.app
                            .attr(
                                &Id::BirthdayInput,
                                Attribute::Text,
                                AttrValue::String(birthday)
                            )
                            .is_ok()
                    );
                    None
                }
                Msg::BirthdayInputBlur => {
                    assert!(self.app.active(&Id::ZecwalletFromMnemonicButton).is_ok());
                    None
                }
                Msg::FromMnemonicSubmitBlur => {
                    assert!(self.app.active(&Id::MnemonicInput).is_ok());
                    None
                }
                Msg::FromPathSubmitBlur => {
                    assert!(self.app.active(&Id::ZecwalletFromPath).is_ok());
                    None
                }
                Msg::FromMnemonicSubmit => {
                    let mnemonic: String = self
                        .app
                        .query(&Id::MnemonicInput, Attribute::Text)
                        .ok()
                        .unwrap()
                        .unwrap()
                        .unwrap_string();
                    let birthday = self
                        .app
                        .query(&Id::BirthdayInput, Attribute::Text)
                        .ok()
                        .flatten()
                        .unwrap()
                        .as_string()
                        .and_then(|s| s.parse::<u32>().ok());

                    Some(Msg::StartSync(SyncSource::Mnemonic {
                        mnemonic: mnemonic.to_string(),
                        birthday,
                    }))
                }
                Msg::FromPathSubmit => {
                    let path: String = self
                        .app
                        .query(&Id::ZecwalletFromPath, Attribute::Text)
                        .ok()
                        .unwrap()
                        .unwrap()
                        .unwrap_string();

                    Some(Msg::StartSync(SyncSource::WalletFile(
                        PathBuf::from_str(&path).unwrap(),
                    )))
                }
                Msg::StartSync(source) => {
                    self.navigate_to(Screen::Syncing);

                    let sv = Arc::clone(&self.sync_view);
                    let lc_lock = Arc::clone(&self.light_client);

                    tokio::spawn(async move {
                        let result_lc: LightClient = match source {
                            SyncSource::WalletFile(path) => {
                                sv.start_wallet_sync_from_path(path).await
                            }
                            SyncSource::Mnemonic { mnemonic, birthday } => {
                                sv.start_wallet_sync_from_mnemonic(mnemonic, birthday).await
                            }
                        };

                        {
                            let mut guard = lc_lock.write().await;
                            *guard = Some(result_lc);
                        }
                    });

                    None
                }
                Msg::GoToResult => {
                    self.navigate_to(Screen::Result);
                    None
                }
                Msg::InitializeLightClient => None,
                Msg::FetchBalance => None,
                Msg::BalanceReady(balance) => {
                    let balance_str = format!("{:?}", balance);
                    let _ = self.app.attr(
                        &Id::ResultViewer,
                        Attribute::Text,
                        AttrValue::Payload(PropPayload::One(PropValue::Str(balance_str))),
                    );
                    self.redraw = true;
                    None
                }
            }
        } else {
            None
        }
    }
}

pub trait HasScreenAndQuit {
    fn navigate_to(&mut self, screen: Screen);
    fn set_quit(&mut self, quit: bool);
}

impl<T: TerminalAdapter> HasScreenAndQuit for Model<T> {
    fn navigate_to(&mut self, screen: Screen) {
        // Update screen
        self.screen = screen;
        // Focus new screen
        match self.screen {
            Screen::MainMenu => {
                let _ = self.app.active(&Id::MainMenu);
            }
            Screen::ZecwalletInput => {
                let _ = self.app.active(&Id::ZecwalletMenu);
            }
            Screen::ZcashdInput => {
                todo!()
            }
            Screen::Syncing => {
                let _ = self.app.active(&Id::SyncLog);
            }
            Screen::ZecwalletFromPath => {
                let _ = self.app.active(&Id::ZecwalletFromPath);
            }
            Screen::ZecwalletFromMnemonic => {
                let _ = self.app.active(&Id::MnemonicInput);
            }
            Screen::Result => {
                let _ = self.app.active(&Id::ExportView);
            }
            Screen::ExportSend => {
                let _ = self.app.active(&Id::ExportSend);
            }
            Screen::ExportZewif => {
                let _ = self.app.active(&Id::ExportZewif);
            }
            Screen::ExportZingolib => {
                let _ = self.app.active(&Id::ExportZingolib);
            }
        }
    }

    fn set_quit(&mut self, quit: bool) {
        self.quit = quit;
    }
}
