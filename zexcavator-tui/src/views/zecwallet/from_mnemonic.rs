use anyhow::Result;
use tuirealm::event::Key;
use tuirealm::props::BorderSides;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::ratatui::text::Text;
use tuirealm::ratatui::widgets::{Block, Paragraph};
use tuirealm::{Application, Component, Event, Frame, MockComponent, NoUserEvent};

use crate::components::birthday_input::BirthdayInput;
use crate::components::log_viewer::{LogBuffer, start_wallet_sync_from_mnemonic};
use crate::components::mnemonic_input::MnemonicInput;
use crate::views::Renderable;
use crate::{Id, Msg};

use super::Mountable;

pub struct ZecwalletFromMnemonic {
    log_buffer: LogBuffer,
}

impl ZecwalletFromMnemonic {
    pub fn start_sync(&mut self, mnemonic: String, birthday: u32) {
        start_wallet_sync_from_mnemonic(self.log_buffer.clone(), mnemonic, birthday);
    }

    pub fn new_with_log(log_buffer: LogBuffer) -> Self {
        Self { log_buffer }
    }

    pub fn validate_input(mnemonic: String) -> Result<()> {
        MnemonicInput::validate_input(mnemonic);
        Ok(())
    }
}

impl Mountable for ZecwalletFromMnemonic {
    fn mount(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()> {
        // Mount mnemonic input
        assert!(
            app.mount(
                Id::MnemonicInput,
                Box::new(MnemonicInput::new(String::new())),
                Vec::default()
            )
            .is_ok()
        );

        // Mount birthday input
        assert!(
            app.mount(
                Id::BirthdayInput,
                Box::new(BirthdayInput::new(String::new())),
                Vec::default()
            )
            .is_ok()
        );

        assert!(
            app.mount(
                Id::ZecwalletFromPathButton,
                Box::new(SubmitButton),
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
                Constraint::Percentage(70),
            ])
            .split(f.area());
        app.view(&Id::MnemonicInput, f, chunks[0]);
        app.view(&Id::BirthdayInput, f, chunks[1]);
        app.view(&Id::ZecwalletFromPathButton, f, chunks[2]);
        app.view(&Id::LogViewerSeed, f, chunks[3]);
    }
}

pub struct SubmitButton;

impl MockComponent for SubmitButton {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let button = Paragraph::new(Text::raw("Submit"))
            .alignment(tuirealm::props::Alignment::Center)
            .block(Block::default().borders(BorderSides::all()));
        frame.render_widget(button, area);
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        todo!()
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {}

    fn state(&self) -> tuirealm::State {
        todo!()
    }

    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        todo!()
    }
}

impl Component<Msg, NoUserEvent> for SubmitButton {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        if let Event::Keyboard(key) = ev {
            match key.code {
                Key::Enter => return Some(Msg::FromMnemonicSubmit),
                Key::Tab => return Some(Msg::FromMnemonicSubmitBlur),
                Key::Esc => return Some(Msg::AppClose),
                _ => (),
            }
        }
        None
    }
}
