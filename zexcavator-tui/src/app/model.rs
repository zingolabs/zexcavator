//! ## Model
//!
//! app model

use std::time::{Duration, SystemTime};

use tuirealm::event::NoUserEvent;
use tuirealm::props::{Alignment, Color, TextModifiers};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::terminal::{CrosstermTerminalAdapter, TerminalAdapter, TerminalBridge};
use tuirealm::{
    Application, AttrValue, Attribute, EventListenerCfg, Sub, SubClause, SubEventClause, Update,
};

use crate::components::SeedInput;

use super::components::{Clock, Label};
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
                        .constraints(
                            [
                                Constraint::Length(3), // Clock
                                Constraint::Length(1), // Label
                                Constraint::Length(3), // Seed input
                            ]
                            .as_ref(),
                        )
                        .split(f.area());
                    self.app.view(&Id::Clock, f, chunks[0]);
                    self.app.view(&Id::Label, f, chunks[1]);
                    self.app.view(&Id::SeedInput, f, chunks[2]);
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
        // Mount components
        assert!(
            app.mount(
                Id::Label,
                Box::new(
                    Label::default()
                        .text("Waiting for a Msg...")
                        .alignment(Alignment::Left)
                        .background(Color::Reset)
                        .foreground(Color::LightYellow)
                        .modifiers(TextModifiers::BOLD),
                ),
                Vec::default(),
            )
            .is_ok()
        );
        // Mount clock, subscribe to tick
        assert!(
            app.mount(
                Id::Clock,
                Box::new(
                    Clock::new(SystemTime::now())
                        .alignment(Alignment::Center)
                        .background(Color::Reset)
                        .foreground(Color::Cyan)
                        .modifiers(TextModifiers::BOLD)
                ),
                vec![Sub::new(SubEventClause::Tick, SubClause::Always)]
            )
            .is_ok()
        );

        // Mount Seed Input
        assert!(
            app.mount(
                Id::SeedInput,
                Box::new(SeedInput::default()),
                Vec::default()
            )
            .is_ok()
        );

        // Active Seed Input
        assert!(app.active(&Id::SeedInput).is_ok());

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
            }
        } else {
            None
        }
    }
}
