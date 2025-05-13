use std::sync::Arc;

use tokio::sync::RwLock;
use tuirealm::event::{Key, KeyEvent};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::ratatui::widgets::{Block, Borders, Paragraph};
use tuirealm::{Component, Frame, MockComponent, NoUserEvent, State};
use zingolib::lightclient::{LightClient, PoolBalances};

use crate::Msg;
use crate::app::model::HasScreenAndQuit;
use crate::components::HandleMessage;

// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
// pub enum ExportOptions {
//     ZeWIF,
//     Send,
// }

// impl MenuOptions for ExportOptions {
//     fn all() -> Vec<Self>
//     where
//         Self: Sized,
//     {
//         vec![Self::ZeWIF, Self::Send]
//     }

//     fn label(&self) -> &'static str {
//         match self {
//             Self::ZeWIF => "ZeWIF",
//             Self::Send => "Send",
//         }
//     }
// }

#[derive(Debug, Clone)]
pub struct ExportSendView {
    // pub light_client: Arc<RwLock<Option<LightClient>>>,
    pub balance: Arc<RwLock<Option<PoolBalances>>>,
    // pub menu: Menu<ExportOptions>,
}

impl ExportSendView {
    pub fn new(_light_client: Arc<RwLock<Option<LightClient>>>) -> Self {
        Self {
            // light_client,
            balance: Arc::new(RwLock::new(None)),
            // menu: Menu::new("Choose an export option"),
        }
    }
}

impl MockComponent for ExportSendView {
    fn view(&mut self, frame: &mut Frame, area: tuirealm::ratatui::prelude::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        // Show balance summary
        if self.balance.try_read().unwrap().is_some() {
            let text = format!("{:?}", self.balance.try_read().unwrap().as_ref().unwrap());
            let para =
                Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Balance"));
            frame.render_widget(para, chunks[0]);
        }

        // self.menu.view(frame, chunks[1]);
    }

    fn query(&self, _attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        todo!()
    }

    fn attr(&mut self, _attr: tuirealm::Attribute, _value: tuirealm::AttrValue) {}

    fn state(&self) -> State {
        todo!()
    }

    fn perform(&mut self, _cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        todo!()
    }
}

impl Component<Msg, NoUserEvent> for ExportSendView {
    fn on(&mut self, ev: tuirealm::Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            tuirealm::Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::AppClose),
            _ => None,
        }
    }
}

impl<T> HandleMessage<T> for ExportSendView
where
    T: HasScreenAndQuit,
{
    fn handle_message(_msg: Msg, _model: &mut T) -> Option<Msg> {
        None
    }
}
