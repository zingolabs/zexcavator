//! ## Model
//!
//! app model

use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use tuirealm::event::NoUserEvent;
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge};
use tuirealm::{Application, AttrValue, Attribute, EventListenerCfg, Update};

use crate::components::HandleMessage;
use crate::components::log_viewer::{LogViewer, new_log_buffer};
use crate::components::menu::MenuOptions;
use crate::views::main_menu::{MainMenu, MainMenuOption};
use crate::views::zecwallet::from_mnemonic::ZecwalletFromMnemonic;
use crate::views::zecwallet::from_path::ZecwalletFromPath;
use crate::views::zecwallet::{ZecwalletMenu, ZecwalletMenuOption};
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
    pub zecwallet_from_path: ZecwalletFromPath,
    pub zecwallet_from_mnemonic: ZecwalletFromMnemonic,
}

impl Default for Model<CrosstermTerminalAdapter> {
    fn default() -> Self {
        let log_buffer_path = new_log_buffer();
        let log_buffer_seed = new_log_buffer();

        let mut app = Self::init_app();

        assert!(
            app.mount(
                Id::LogViewerPath,
                Box::new(LogViewer::new(log_buffer_path.clone())),
                Vec::default()
            )
            .is_ok()
        );

        assert!(
            app.mount(
                Id::LogViewerSeed,
                Box::new(LogViewer::new(log_buffer_seed.clone())),
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
            zecwallet_from_path: ZecwalletFromPath::new_with_log(log_buffer_path),
            zecwallet_from_mnemonic: ZecwalletFromMnemonic::new_with_log(log_buffer_seed),
        }
    }
}

impl<T> Model<T>
where
    T: TerminalAdapter,
{
    pub fn view(&mut self) {
        let _screen = self.screen;
        let app = &mut self.app;
        assert!(
            self.terminal
                .draw(|f| {
                    match self.screen {
                        Screen::MainMenu => main_menu::render(app, f),
                        Screen::Syncing => todo!(),
                        Screen::ZecwalletInput => ZecwalletMenu::render(app, f),
                        Screen::ZecwalletFromPath => ZecwalletFromPath::render(app, f),
                        Screen::ZecwalletFromMnemonic => ZecwalletFromMnemonic::render(app, f),
                        Screen::ZcashdInput => todo!(),
                    }
                })
                .is_ok()
        );
    }

    fn init_app() -> Application<Id, Msg, NoUserEvent> {
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

        let log_buffer = new_log_buffer();

        // Create the screen and give it the buffer
        ZecwalletFromPath::new_with_log(log_buffer.clone());

        assert!(ZecwalletFromPath::mount(&mut app).is_ok());

        // Create the screen and give it the buffer
        ZecwalletFromMnemonic::new_with_log(log_buffer);

        assert!(ZecwalletFromMnemonic::mount(&mut app).is_ok());

        // Active main menu
        assert!(app.active(&Id::MainMenu).is_ok());

        app
    }
}

// Let's implement Update for model
impl<T> Update<Msg> for Model<T>
where
    T: TerminalAdapter,
{
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            // Set redraw
            self.redraw = true;
            // Match message
            match msg {
                Msg::AppClose => {
                    self.quit = true; // Terminate
                    None
                }
                Msg::SeedInputBlur => None,
                Msg::SeedInputChanged(s) => {
                    assert!(
                        self.app
                            .attr(
                                &Id::ZecwalletFromPath,
                                Attribute::Text,
                                AttrValue::String(format!("LetterCounter has now value: {}", s))
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
                            self.zecwallet_from_path.start_sync(path);
                            return None;
                        }
                    };
                    None
                }
                Msg::Start => {
                    self.screen = Screen::MainMenu;
                    None
                }
                Msg::MenuCursorMove(_) => None,
                Msg::MenuSelected(label) => {
                    match (
                        MainMenuOption::from_label(&label),
                        ZecwalletMenuOption::from_label(&label),
                    ) {
                        (Some(_), _) => MainMenu::handle_message(Msg::MenuSelected(label), self),
                        (_, Some(_)) => {
                            ZecwalletMenu::handle_message(Msg::MenuSelected(label), self)
                        }
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
                Msg::MnemonicInputValidate => {
                    let mnemonic = self
                        .app
                        .query(&Id::MnemonicInput, Attribute::Text)
                        .unwrap()
                        .unwrap()
                        .as_string()
                        .unwrap()
                        .to_string();
                    let birthday = self
                        .app
                        .query(&Id::BirthdayInput, Attribute::Text)
                        .unwrap()
                        .unwrap()
                        .as_number()
                        .unwrap() as u32;
                    match ZecwalletFromMnemonic::validate_input(mnemonic.clone()) {
                        Err(_) => None::<Msg>,
                        Ok(_) => {
                            self.zecwallet_from_mnemonic.start_sync(mnemonic, birthday);

                            return None;
                        }
                    };

                    None
                }
                Msg::MnemonicInputBlur => {
                    assert!(self.app.active(&Id::BirthdayInput).is_ok());
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
                    assert!(self.app.active(&Id::ZecwalletFromPathButton).is_ok());
                    None
                }
                Msg::FromMnemonicSubmitBlur => {
                    assert!(self.app.active(&Id::MnemonicInput).is_ok());
                    None
                }
                Msg::FromMnemonicSubmit => {
                    let mnemonic = self
                        .app
                        .query(&Id::MnemonicInput, Attribute::Text)
                        .unwrap()
                        .unwrap()
                        .as_string()
                        .unwrap()
                        .to_string();
                    let birthday_str = self.app.query(&Id::BirthdayInput, Attribute::Text);

                    let birthday = birthday_str
                        .unwrap()
                        .unwrap()
                        .as_string()
                        .unwrap()
                        .trim()
                        .parse()
                        .unwrap();
                    match ZecwalletFromMnemonic::validate_input(mnemonic.clone()) {
                        Err(_) => None::<Msg>,
                        Ok(_) => {
                            self.zecwallet_from_mnemonic.start_sync(mnemonic, birthday);

                            return None;
                        }
                    };

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
        // Blur current active screen
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
                todo!()
            }
            Screen::ZecwalletFromPath => {
                let _ = self.app.active(&Id::ZecwalletFromPath);
            }
            Screen::ZecwalletFromMnemonic => {
                let _ = self.app.active(&Id::ZecwalletFromMnemonic);
            }
        }

        // Update screen
        self.screen = screen;

        // Activate new screen
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
            Screen::ZecwalletFromPath => {
                // TODO: It should be more clear that this refers to the input inside the ZecwalletFromPath view.
                let _ = self.app.active(&Id::ZecwalletFromPath);
            }
            Screen::ZecwalletFromMnemonic => {
                let _ = self.app.active(&Id::MnemonicInput);
            }
            Screen::Syncing => {
                todo!()
            }
        }
    }

    fn set_quit(&mut self, quit: bool) {
        self.quit = quit;
    }
}
