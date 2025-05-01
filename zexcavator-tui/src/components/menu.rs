use tuirealm::Component;
use tuirealm::MockComponent;
use tuirealm::NoUserEvent;
use tuirealm::State;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::event::{Event, Key};
use tuirealm::props::TextModifiers;
use tuirealm::props::{AttrValue, Attribute};

use crate::Msg;

/// TODO: Maybe use `strum` here?
pub trait MenuOptions {
    fn all() -> Vec<Self>
    where
        Self: Sized;

    fn label(&self) -> &'static str;

    fn from_label(label: &str) -> Option<Self>
    where
        Self: Sized,
    {
        Self::all().into_iter().find(|opt| opt.label() == label)
    }
}

pub struct Menu<T: MenuOptions + Clone> {
    options: Vec<T>,
    cursor_position: usize,
    title: String,
}

impl<T: MenuOptions + Clone> Menu<T> {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            options: T::all(),
            cursor_position: 0,
        }
    }
}

impl<T: MenuOptions + Clone> MockComponent for Menu<T> {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: tuirealm::ratatui::layout::Rect) {
        use tuirealm::ratatui::text::{Line, Span, Text};
        use tuirealm::ratatui::widgets::{Block, Borders, Paragraph};

        let lines: Vec<Line> = self
            .options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                if i == self.cursor_position {
                    Line::from(Span::styled(
                        format!("> {}", option.label()),
                        tuirealm::props::Style::default()
                            .fg(tuirealm::props::Color::Yellow)
                            .add_modifier(TextModifiers::REVERSED),
                    ))
                } else {
                    Line::from(Span::raw(format!("  {}", option.label())))
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

impl<T: MenuOptions + Clone> Component<Msg, NoUserEvent> for Menu<T> {
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
                Key::Enter => {
                    let selected = self.options[self.cursor_position].clone();
                    Some(Msg::MenuSelected(selected.label().to_string()))
                }
                Key::Esc => Some(Msg::AppClose),
                _ => None,
            },
            _ => None,
        }
    }
}
