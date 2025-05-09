use tuirealm::event::Key;

pub struct ResultViewer {
    output: Vec<String>,
}

impl ResultViewer {
    pub fn new(output: Vec<String>) -> Self {
        Self { output }
    }
}

use tuirealm::props::{PropPayload, PropValue};
use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::widgets::Wrap;
use tuirealm::{Component, Event, Frame, MockComponent, NoUserEvent};

use crate::Msg;

impl MockComponent for ResultViewer {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        use tuirealm::ratatui::text::{Line, Span, Text};
        use tuirealm::ratatui::widgets::{Block, Borders, Paragraph};

        let log_lines = self.output.clone();
        let text = Text::from(
            log_lines
                .iter()
                .map(|l| Line::from(Span::raw(l)))
                .collect::<Vec<_>>(),
        );

        let scroll_offset = log_lines.len().saturating_sub(area.height as usize - 2);

        let paragraph = Paragraph::new(text)
            .block(Block::default().title("Sync Log").borders(Borders::ALL))
            .scroll((scroll_offset as u16, 0))
            .wrap(Wrap { trim: true });

        frame.render_widget(paragraph, area);
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        todo!()
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        match attr {
            tuirealm::Attribute::Value => {
                if let tuirealm::AttrValue::Payload(PropPayload::One(PropValue::Str(msg))) = value {
                    self.output.push(msg);
                }
            }
            _ => (),
        }
    }

    fn state(&self) -> tuirealm::State {
        todo!()
    }

    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        todo!()
    }
}

impl Component<Msg, NoUserEvent> for ResultViewer {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        if let Event::Keyboard(key) = ev {
            if key.code == Key::Esc {
                return Some(Msg::AppClose);
            }
        }
        None
    }
}
