use std::sync::Arc;

use tokio::sync::RwLock;
use tuirealm::event::{Key, KeyEvent};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::ratatui::widgets::{Block, Borders, Paragraph};
use tuirealm::{Component, Frame, MockComponent, NoUserEvent, State};
use zingolib::lightclient::LightClient;
use zingolib::wallet::balance::AccountBalance;

use crate::Msg;
use crate::app::model::HasScreenAndQuit;
use crate::components::HandleMessage;

#[derive(Debug, Clone)]
pub struct ExportSendView {
    // pub light_client: Arc<RwLock<Option<LightClient>>>,
    pub balance: Arc<RwLock<Option<AccountBalance>>>,
}

impl ExportSendView {
    pub fn new(_light_client: Arc<RwLock<Option<LightClient>>>) -> Self {
        Self {
            balance: Arc::new(RwLock::new(None)),
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
            tuirealm::Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::GoToResult),
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
