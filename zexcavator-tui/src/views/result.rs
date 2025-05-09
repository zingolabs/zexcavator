use std::sync::{Arc, Mutex};

use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::{Application, Frame, NoUserEvent, State};
use zingolib::lightclient::{LightClient, PoolBalances};

use crate::app::model::HasScreenAndQuit;
use crate::components::HandleMessage;
use crate::components::result_viewer::ResultViewer;
use crate::{Id, Msg};

use crate::components::menu::{Menu, MenuOptions};

use super::{Mountable, Renderable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportOptions {
    ZeWIF,
    Send,
}

impl MenuOptions for ExportOptions {
    fn all() -> Vec<Self>
    where
        Self: Sized,
    {
        vec![Self::ZeWIF, Self::Send]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::ZeWIF => "ZeWIF",
            Self::Send => "Send",
        }
    }
}

pub struct ExportMenu {
    pub light_client: Arc<Mutex<Option<LightClient>>>,
    pub balance: Arc<Mutex<Option<PoolBalances>>>,
}

impl ExportMenu {
    pub fn new(light_client: Arc<Mutex<Option<LightClient>>>) -> Self {
        Self {
            light_client,
            balance: Arc::new(Mutex::new(None)),
        }
    }

    // pub fn get_balance(&self) -> Option<PoolBalances> {
    // let light_client = Arc::clone(&self.light_client);
    // let balance = Arc::clone(&self.balance);

    // tokio::spawn(async move {
    //     let light_client_instance = {
    //         let mut lc_guard = light_client.lock().unwrap();
    //         lc_guard.take()
    //     };

    //     if let Some(lc) = light_client_instance {
    //         let b = lc.do_balance().await;
    //         let mut balance_guard = balance.lock().unwrap();
    //         *balance_guard = Some(b);

    //         println!("Balance successfully fetched");
    //     } else {
    //         println!("No LightClient available when trying to fetch balance");
    //     }
    // });
    // let lc_guard = self.light_client.lock().unwrap();
    // if let Some(lc) = lc_guard.as_ref() {
    //     Some(lc.do_balance())
    // } else {
    //     None
    // }
    // }
}

impl Mountable for ExportMenu {
    fn mount(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()> {
        // Mount result viewer
        assert!(
            app.mount(
                Id::ResultViewer,
                Box::new(ResultViewer::new(Vec::default())),
                Vec::default()
            )
            .is_ok()
        );

        // Mount menu
        assert!(
            app.mount(
                Id::ExportMenu,
                Box::new(Menu::<ExportOptions>::new("Select the export method",)),
                Vec::default()
            )
            .is_ok()
        );
        Ok(())
    }
}

impl Renderable for ExportMenu {
    fn render(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Percentage(50), Constraint::Length(20)])
            .split(f.area());

        app.view(&Id::ResultViewer, f, chunks[0]);
        app.view(&Id::ExportMenu, f, chunks[1]);
    }
}

impl<T> HandleMessage<T> for ExportMenu
where
    T: HasScreenAndQuit,
{
    fn handle_message(msg: Msg, model: &mut T) -> Option<Msg> {
        match msg {
            Msg::MenuSelected(option) => {
                if let Some(menu_item) = ExportOptions::from_label(&option) {
                    match menu_item {
                        ExportOptions::ZeWIF => {
                            // model.navigate_to(Screen::ZecwalletFromPath)
                            todo!()
                        }
                        ExportOptions::Send => {
                            // model.navigate_to(Screen::ZecwalletFromMnemonic)
                            todo!()
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}
