use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::{Application, Frame, NoUserEvent};

use crate::app::model::{HasScreenAndQuit, Screen};
use crate::components::HandleMessage;
use crate::components::log_viewer::{LogViewer, new_log_buffer};
use crate::{Id, Msg};

use crate::components::menu::{Menu, MenuOptions};

use super::Mountable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZecwalletMenuOption {
    Mnemonic,
    Path,
    Seed,
}

impl MenuOptions for ZecwalletMenuOption {
    fn all() -> Vec<Self>
    where
        Self: Sized,
    {
        vec![Self::Mnemonic, Self::Path, Self::Seed]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Mnemonic => "From Mnemonic",
            Self::Path => "From Path",
            Self::Seed => "From Seed",
        }
    }
}

pub struct ZecwalletMenu;

pub fn render(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(20), Constraint::Length(20)])
        .split(f.area());
    app.view(&Id::ZecwalletMenu, f, chunks[0]);
    app.view(&Id::LogViewer, f, chunks[1]);
}

impl Mountable for ZecwalletMenu {
    fn mount(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()> {
        let options: Vec<&str> = ZecwalletMenuOption::all()
            .into_iter()
            .map(|opt| opt.label())
            .collect();

        // Mount logo
        assert!(
            app.mount(
                Id::ZecwalletMenu,
                Box::new(Menu::<ZecwalletMenuOption>::new("Select the input method",)),
                Vec::default()
            )
            .is_ok()
        );

        // Mount main menu
        assert!(
            app.mount(
                Id::LogViewer,
                Box::new(LogViewer::new(new_log_buffer())),
                Vec::default(),
            )
            .is_ok()
        );
        Ok(())
    }
}

impl<T> HandleMessage<T> for ZecwalletMenu
where
    T: HasScreenAndQuit,
{
    fn handle_message(msg: Msg, model: &mut T) -> Option<Msg> {
        match msg {
            Msg::MenuSelected(option) => {
                if let Some(menu_item) = ZecwalletMenuOption::from_label(&option) {
                    match menu_item {
                        ZecwalletMenuOption::Mnemonic => model.navigate_to(Screen::ZecwalletInput),
                        ZecwalletMenuOption::Path => model.navigate_to(Screen::ZecwalletInput),
                        ZecwalletMenuOption::Seed => model.navigate_to(Screen::ZecwalletInput),
                    }
                }
                None
            }
            _ => None,
        }
    }
}
