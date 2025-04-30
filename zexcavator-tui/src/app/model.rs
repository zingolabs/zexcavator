//! ## Model
//!
//! app model

use std::time::Duration;

use tuirealm::event::NoUserEvent;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge};
use tuirealm::{Application, AttrValue, Attribute, EventListenerCfg, Update};

use crate::components::{LogViewer, MainMenu, WelcomeComponent, new_log_buffer, start_wallet_sync};

use super::{Id, Msg};

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
}

impl Default for Model<CrosstermTerminalAdapter> {
    fn default() -> Self {
        Self {
            app: Self::init_app(),
            quit: false,
            redraw: true,
            terminal: TerminalBridge::init_crossterm().expect("Cannot initialize terminal"),
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
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(1)
                        .constraints(&[
                            Constraint::Length(20), // Welcome component
                            Constraint::Length(10), // Main menu
                            Constraint::Length(30), // Log viewer
                        ])
                        .split(f.area());
                    self.app.view(&Id::WelcomeComponent, f, chunks[0]);
                    self.app.view(&Id::MainMenu, f, chunks[1]);
                    self.app.view(&Id::LogViewer, f, chunks[2]);
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
        // Mount welcome screen!
        assert!(
            app.mount(
                Id::WelcomeComponent,
                Box::new(WelcomeComponent),
                Vec::default()
            )
            .is_ok()
        );

        // Mount main menu
        assert!(
            app.mount(
                Id::MainMenu,
                Box::new(MainMenu::new(
                    "Select the wallet to recover",
                    vec![
                        "Zecwallet",
                        "zcashd",
                        "Ledger",
                        "Trezor",
                        "Porygon-Z",
                        "Ekans",
                        "Charmander"
                    ],
                )),
                Vec::default()
            )
            .is_ok()
        );

        let log_buffer = new_log_buffer();
        start_wallet_sync(log_buffer.clone());

        // Mount log viewer
        assert!(
            app.mount(
                Id::LogViewer,
                Box::new(LogViewer::new(log_buffer)),
                Vec::default()
            )
            .is_ok()
        );

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
                Msg::Clock => None,
                Msg::SeedInputBlur => None,
                Msg::SeedInputChanged(s) => {
                    assert!(
                        self.app
                            .attr(
                                &Id::SeedInput,
                                Attribute::Text,
                                AttrValue::String(format!("LetterCounter has now value: {}", s))
                            )
                            .is_ok()
                    );
                    None
                }
                Msg::None => None,
                Msg::SeedInputValidate(s) => todo!(),
                Msg::Start => todo!(),
                Msg::MenuCursorMove(_) => None,
                Msg::MenuSelected(selection) => {
                    // TODO: Menu items should be declared as enum
                    match selection.as_str() {
                        "Charmander" | "Ekans" | "Porygon-Z" => {
                            // Pokemon selected!
                            self.quit = true;
                        }
                        "Zecwallet" => {
                            // Open recovery flow for zecwallet
                        }
                        "zcashd" => {
                            // Open recovery flow for zcashd
                        }
                        _ => {}
                    }
                    None
                }
            }
        } else {
            None
        }
    }
}
