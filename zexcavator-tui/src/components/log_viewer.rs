use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tuirealm::command::CmdResult;
use tuirealm::event::Key;

pub type LogBuffer = Arc<Mutex<Vec<String>>>;

#[derive(Debug, Clone, PartialEq)]
pub enum SyncSource {
    WalletFile(PathBuf),
    Mnemonic {
        mnemonic: String,
        birthday: Option<u32>,
    },
}

pub fn new_log_buffer() -> LogBuffer {
    Arc::new(Mutex::new(Vec::new()))
}

pub struct LogViewer {
    logs: LogBuffer,
}

impl LogViewer {
    pub fn new(logs: LogBuffer) -> Self {
        Self { logs }
    }
}

use tuirealm::ratatui::layout::Rect;
use tuirealm::ratatui::widgets::Wrap;
use tuirealm::{Component, Event, Frame, MockComponent, NoUserEvent, State};

use crate::Msg;

impl MockComponent for LogViewer {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        use tuirealm::ratatui::text::{Line, Span, Text};
        use tuirealm::ratatui::widgets::{Block, Borders, Paragraph};

        let logs_guard = self.logs.lock();

        if let Ok(log_lines) = logs_guard {
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
    }

    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        None
    }

    fn attr(&mut self, _attr: tuirealm::Attribute, _value: tuirealm::AttrValue) {}

    fn state(&self) -> tuirealm::State {
        State::None
    }

    fn perform(&mut self, _cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for LogViewer {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        if let Event::Keyboard(key) = ev {
            if key.code == Key::Esc {
                return Some(Msg::AppClose);
            }
        }
        None
    }
}
