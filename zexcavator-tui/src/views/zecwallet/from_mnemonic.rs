
use anyhow::Result;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::{Application, Frame, NoUserEvent};

use crate::components::Focusable;
use crate::components::birthday_input::BirthdayInput;
use crate::components::log_viewer::LogBuffer;
use crate::components::mnemonic_input::MnemonicInput;
use crate::views::Renderable;
use crate::{Id, Msg};

use super::Mountable;

pub struct ZecwalletFromMnemonic {
    log_buffer: LogBuffer,
}

impl ZecwalletFromMnemonic {
    pub fn start_sync(&mut self, mnemonic: String, birthday: u32) {
        // start_wallet_sync_from_mnemonic(self.log_buffer.clone(), mnemonic, birthday);
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
        Ok(())
    }
}

impl Renderable for ZecwalletFromMnemonic {
    fn render(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(20),
                Constraint::Percentage(60),
            ])
            .split(f.area());
        app.view(&Id::MnemonicInput, f, chunks[0]);
        app.view(&Id::BirthdayInput, f, chunks[1]);
        app.view(&Id::LogViewerSeed, f, chunks[2]);
    }
}

impl Focusable for ZecwalletFromMnemonic {
    fn on_focus(&mut self) {}
}
