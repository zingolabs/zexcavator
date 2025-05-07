use std::path::PathBuf;

use anyhow::Result;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::{Application, Frame, NoUserEvent};

use crate::components::input::SeedInput;
use crate::views::Renderable;
use crate::{Id, Msg};

use super::Mountable;

#[derive(Default)]
pub struct ZecwalletFromPath;

impl ZecwalletFromPath {
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
            .constraints([Constraint::Percentage(20)])
            .split(f.area());
        app.view(&Id::ZecwalletFromPath, f, chunks[0]);
    }
}
