pub mod from_mnemonic;
pub mod from_path;

use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::{Application, Frame, NoUserEvent};

use crate::app::model::{HasScreenAndQuit, Screen};
use crate::components::HandleMessage;
use crate::{Id, Msg};

use crate::components::menu::{Menu, MenuOptions};

use super::{Mountable, Renderable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZecwalletMenuOption {
    Mnemonic,
    Path,
    Seed,
    Back,
}

impl MenuOptions for ZecwalletMenuOption {
    fn all() -> Vec<Self>
    where
        Self: Sized,
    {
        vec![Self::Mnemonic, Self::Path, Self::Seed, Self::Back]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Mnemonic => "From Mnemonic",
            Self::Path => "From Path",
            Self::Seed => "From Seed (Not yet implemented)",
            Self::Back => "Back",
        }
    }
}

pub struct ZecwalletMenu;

impl Mountable for ZecwalletMenu {
    fn mount(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()> {
        assert!(
            app.mount(
                Id::ZecwalletMenu,
                Box::new(Menu::<ZecwalletMenuOption>::new("Select the input method",)),
                Vec::default()
            )
            .is_ok()
        );
        Ok(())
    }
}

impl Renderable for ZecwalletMenu {
    fn render(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([Constraint::Length(20)])
            .split(f.area());
        app.view(&Id::ZecwalletMenu, f, chunks[0]);
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
                        ZecwalletMenuOption::Path => model.navigate_to(Screen::ZecwalletFromPath),
                        ZecwalletMenuOption::Mnemonic => {
                            model.navigate_to(Screen::ZecwalletFromMnemonic)
                        }
                        ZecwalletMenuOption::Seed => model.navigate_to(Screen::ZecwalletInput),
                        ZecwalletMenuOption::Back => model.navigate_to(Screen::MainMenu),
                    }
                }
                None
            }
            _ => None,
        }
    }
}
