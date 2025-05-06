use std::path::PathBuf;

use anyhow::Result;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::{Application, Frame, NoUserEvent};

use crate::components::Focusable;
use crate::components::input::SeedInput;
use crate::components::log_viewer::{LogBuffer, start_wallet_sync};
use crate::views::Renderable;
use crate::{Id, Msg};

use super::Mountable;

pub struct ZecwalletFromPath {
    log_buffer: LogBuffer,
}

impl ZecwalletFromPath {
    pub fn start_sync(&mut self, path: String) {
        start_wallet_sync(self.log_buffer.clone(), PathBuf::from(path));
    }

    pub fn new_with_log(log_buffer: LogBuffer) -> Self {
        Self { log_buffer }
    }

    pub fn validate_path(path: PathBuf) -> Result<()> {
        path.canonicalize()?;
        Ok(())
    }
}

impl Mountable for ZecwalletFromPath {
    fn mount(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()> {
        // Mount input
        assert!(
            app.mount(
                Id::ZecwalletFromPath,
                Box::new(SeedInput::new(String::new())),
                Vec::default()
            )
            .is_ok()
        );
        Ok(())
    }
}

impl Renderable for ZecwalletFromPath {
    fn render(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(f.area());
        app.view(&Id::ZecwalletFromPath, f, chunks[0]);
        app.view(&Id::LogViewerPath, f, chunks[1]);
    }
}

impl Focusable for ZecwalletFromPath {
    fn on_focus(&mut self) {}
}
