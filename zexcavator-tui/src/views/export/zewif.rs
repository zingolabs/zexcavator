use std::sync::{Arc, Mutex};

use tokio::sync::RwLock;
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::{Component, Frame, MockComponent, NoUserEvent, State};
use zewif::Zewif;
use zingolib::lightclient::{LightClient, PoolBalances};

use crate::Msg;
use crate::app::model::HasScreenAndQuit;
use crate::components::HandleMessage;

// use crate::components::menu::{Menu, MenuOptions};

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
pub struct ExportZewifView {
    pub light_client: Arc<RwLock<Option<LightClient>>>,
    pub saved_path: Arc<Mutex<Option<String>>>,
}

impl ExportZewifView {
    pub fn new(light_client: Arc<RwLock<Option<LightClient>>>) -> Self {
        Self {
            light_client,
            saved_path: Arc::new(Mutex::new(None)),
        }
    }

    /// Converts the LightClient into a ZeWIF-compatible format and saves it to disk
    pub async fn do_save(&self) -> anyhow::Result<String> {
        // Client already synced at this point
        let guard = self.light_client.read().await;
        let lc = guard.as_ref().ok_or_else(|| anyhow::anyhow!("no client"))?;
        // Conversion here
        let zewif = todo!();
        let path = std::env::temp_dir().join("wallet.zewif");
        // tokio::fs::write(&path, zewif).await?;
        // Ok(path.to_string_lossy().into_owned())
    }

    /// Inline implementation of zingolib's LichClient to ZeWIF conversion.
    /// Eventually, this will be moved to the `zewif-zingolib` crate.
    pub fn lc_to_zewif(lc: LightClient) -> Zewif {
        todo!()
    }
}

impl MockComponent for ExportZewifView {
    fn view(&mut self, frame: &mut Frame, area: tuirealm::ratatui::prelude::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)])
            .split(area);

        // // Show balance summary
        // if self.balance.try_read().unwrap().is_some() {
        //     let text = format!("{:?}", self.balance.try_read().unwrap().as_ref().unwrap());
        //     let para =
        //         Paragraph::new(text).block(Block::default().borders(Borders::ALL).title("Balance"));
        //     frame.render_widget(para, chunks[0]);
        // }

        // self.menu.view(frame, chunks[1]);
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        todo!()
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {}

    fn state(&self) -> State {
        todo!()
    }

    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        todo!()
    }
}

impl Component<Msg, NoUserEvent> for ExportZewifView {
    fn on(&mut self, ev: tuirealm::Event<NoUserEvent>) -> Option<Msg> {
        // if let Some(menu_msg) = self.menu.on(ev.clone()) {
        //     return Some(menu_msg);
        // }
        match ev {
            tuirealm::Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::AppClose),
            _ => None,
        }
    }
}

impl<T> HandleMessage<T> for ExportZewifView
where
    T: HasScreenAndQuit,
{
    fn handle_message(msg: Msg, model: &mut T) -> Option<Msg> {
        match msg {
            // Msg::MenuSelected(option) => {
            //     if let Some(menu_item) = ExportOptions::from_label(&option) {
            //         match menu_item {
            //             ExportOptions::ZeWIF => {
            //                 // model.navigate_to(Screen::ZecwalletFromPath)
            //                 todo!()
            //             }
            //             ExportOptions::Send => {
            //                 // model.navigate_to(Screen::ZecwalletFromMnemonic)
            //                 todo!()
            //             }
            //         }
            //     }
            //     None
            // }
            _ => None,
        }
    }
}
