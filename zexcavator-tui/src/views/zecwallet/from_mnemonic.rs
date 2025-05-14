use tuirealm::command::CmdResult;
use tuirealm::event::Key;
use tuirealm::props::BorderSides;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::ratatui::text::Text;
use tuirealm::ratatui::widgets::{Block, Paragraph};
use tuirealm::{Application, Component, Event, Frame, MockComponent, NoUserEvent, State};

use crate::components::birthday_input::BirthdayInput;
use crate::components::mnemonic_input::MnemonicInput;
use crate::views::Renderable;
use crate::{Id, Msg};

use super::Mountable;

#[derive(Default)]
pub struct ZecwalletFromMnemonic;

impl Mountable for ZecwalletFromMnemonic {
    fn mount(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()> {
        // Mount mnemonic input
        assert!(
            app.mount(
                Id::MnemonicInput,
                Box::new(MnemonicInput::new(String::new(), "Mnemonic".to_string())),
                Vec::default()
            )
            .is_ok()
        );

        // Mount birthday input
        assert!(
            app.mount(
                Id::BirthdayInput,
                Box::new(BirthdayInput::new(
                    String::new(),
                    "Wallet birthday".to_string()
                )),
                Vec::default()
            )
            .is_ok()
        );

        // Mount submit button
        assert!(
            app.mount(
                Id::ZecwalletFromMnemonicButton,
                Box::new(SubmitButtonMnemonic),
                Vec::default()
            )
            .is_ok()
        );
        Ok(())
    }
}

impl Renderable for ZecwalletFromMnemonic {
    fn render(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(10),
                Constraint::Percentage(10),
                Constraint::Length(3),
            ])
            .split(f.area());
        app.view(&Id::MnemonicInput, f, chunks[0]);
        app.view(&Id::BirthdayInput, f, chunks[1]);
        app.view(&Id::ZecwalletFromMnemonicButton, f, chunks[2]);
    }
}

pub struct SubmitButtonMnemonic;

impl MockComponent for SubmitButtonMnemonic {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let button = Paragraph::new(Text::raw("Submit"))
            .alignment(tuirealm::props::Alignment::Center)
            .block(Block::default().borders(BorderSides::all()));
        frame.render_widget(button, area);
    }

    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        None
    }

    fn attr(&mut self, _attr: tuirealm::Attribute, _value: tuirealm::AttrValue) {}

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for SubmitButtonMnemonic {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        if let Event::Keyboard(key) = ev {
            match key.code {
                Key::Enter => {
                    return Some(Msg::FromMnemonicSubmit);
                }
                Key::Tab => return Some(Msg::FromMnemonicSubmitBlur),
                Key::Esc => return Some(Msg::AppClose),
                _ => (),
            }
        }
        None
    }
}
