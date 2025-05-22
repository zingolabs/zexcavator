use std::path::PathBuf;

use anyhow::Result;
use tuirealm::command::CmdResult;
use tuirealm::event::Key;
use tuirealm::props::BorderSides;
use tuirealm::ratatui::layout::{Constraint, Direction, Layout, Rect};
use tuirealm::ratatui::text::Text;
use tuirealm::ratatui::widgets::{Block, Paragraph};
use tuirealm::{
    Application, AttrValue, Attribute, Component, Event, Frame, MockComponent, NoUserEvent, State,
};

use crate::components::input::PathInput;
use crate::constants::colors::ZINGO_GREEN;
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
                Box::new(PathInput::new(String::new(), "Wallet location".to_string())),
                Vec::default()
            )
            .is_ok()
        );

        // Mount submit button
        assert!(
            app.mount(
                Id::ZecwalletFromPathButton,
                Box::new(SubmitButtonPath::default()),
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

#[derive(Default)]
pub struct SubmitButtonPath {
    focused: bool,
}

impl MockComponent for SubmitButtonPath {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let border_style = if self.focused {
            tuirealm::ratatui::style::Style::default().fg(ZINGO_GREEN)
        } else {
            tuirealm::ratatui::style::Style::default()
        };

        let button = Paragraph::new(Text::raw("Submit"))
            .alignment(tuirealm::props::Alignment::Center)
            .block(
                Block::default()
                    .borders(BorderSides::all())
                    .border_style(border_style),
            );

        frame.render_widget(button, area);
    }

    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        None
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        if attr == Attribute::Focus {
            if let AttrValue::Flag(focus_flag) = value {
                self.focused = focus_flag;
            }
        }
    }

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, _cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        CmdResult::None
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
                Key::Esc => return Some(Msg::Start),
                _ => (),
            }
        }
        None
    }
}
