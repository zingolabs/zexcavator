use std::path::PathBuf;

use anyhow::Result;
use tuirealm::event::Key;
use tuirealm::props::BorderSides;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::ratatui::text::Text;
use tuirealm::ratatui::widgets::{Block, Paragraph};
use tuirealm::{Application, Component, Event, Frame, MockComponent, NoUserEvent};

use crate::components::input::PathInput;
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
                Box::new(PathInput::new(String::new())),
                Vec::default()
            )
            .is_ok()
        );

        // Mount submit button
        assert!(
            app.mount(
                Id::ZecwalletFromPathButton,
                Box::new(SubmitButtonPath),
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
            .constraints([Constraint::Percentage(20), Constraint::Length(3)])
            .split(f.area());
        app.view(&Id::ZecwalletFromPath, f, chunks[0]);
        app.view(&Id::ZecwalletFromPathButton, f, chunks[1]);
    }
}

pub struct SubmitButtonPath;

impl MockComponent for SubmitButtonPath {
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

impl Component<Msg, NoUserEvent> for SubmitButtonPath {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        if let Event::Keyboard(key) = ev {
            match key.code {
                Key::Enter => {
                    return Some(Msg::FromPathSubmit);
                }
                Key::Tab => return Some(Msg::FromPathSubmitBlur),
                Key::Esc => return Some(Msg::AppClose),
                _ => (),
            }
        }
        None
    }
}
