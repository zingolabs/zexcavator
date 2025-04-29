use tuirealm::Component;
use tuirealm::MockComponent;
use tuirealm::NoUserEvent;
use tuirealm::State;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Event, Key};
use tuirealm::props::TextModifiers;
use tuirealm::props::{AttrValue, Attribute};

use crate::Msg;

pub struct MainMenu {
    options: Vec<String>,
    cursor_position: usize,
    title: String,
}

impl MainMenu {
    pub fn new(title: &str, options: Vec<&'static str>) -> Self {
        Self {
            title: title.to_string(),
            options: options.iter().map(|s| s.to_string()).collect(),
            cursor_position: 0,
        }
    }
}

impl MockComponent for MainMenu {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: tuirealm::ratatui::layout::Rect) {
        use tuirealm::ratatui::text::{Line, Span, Text};
        use tuirealm::ratatui::widgets::{Block, Borders, Paragraph};

        let lines: Vec<Line> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, label)| {
                if i == self.cursor_position {
                    Line::from(Span::styled(
                        format!("> {}", label),
                        tuirealm::props::Style::default()
                            .fg(tuirealm::props::Color::Yellow)
                            .add_modifier(TextModifiers::REVERSED),
                    ))
                } else {
                    Line::from(Span::raw(format!("  {}", label)))
                }
            })
            .collect();

        let block = Paragraph::new(Text::from(lines)).block(
            Block::default()
                .title(self.title.clone())
                .borders(Borders::ALL),
        );

        frame.render_widget(block, area);
    }

    fn query(&self, _attr: Attribute) -> Option<AttrValue> {
        None
    }
    fn attr(&mut self, _attr: Attribute, _value: AttrValue) {}
    fn state(&self) -> State {
        State::None
    }
    fn perform(&mut self, _cmd: Cmd) -> CmdResult {
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for MainMenu {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(key) => match key.code {
                Key::Up => {
                    if self.cursor_position == 0 {
                        self.cursor_position = self.options.len() - 1;
                    } else {
                        self.cursor_position -= 1;
                    }
                    Some(Msg::MenuCursorMove(self.cursor_position))
                }
                Key::Down => {
                    self.cursor_position = (self.cursor_position + 1) % self.options.len();
                    Some(Msg::MenuCursorMove(self.cursor_position))
                }
                Key::Enter => Some(Msg::MenuSelected(
                    self.options[self.cursor_position].clone(),
                )),
                Key::Esc => Some(Msg::AppClose),
                _ => None,
            },
            _ => None,
        }
    }
}
