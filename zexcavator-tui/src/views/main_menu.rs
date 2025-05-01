use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::{Application, Frame, NoUserEvent};

use crate::app::model::{HasScreenAndQuit, Screen};
use crate::components::HandleMessage;
use crate::components::menu::{Menu, MenuOptions};
use crate::components::welcome::WelcomeComponent;
use crate::{Id, Msg};

use super::Mountable;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuOption {
    Zecwallet,
    Zcashd,
    Ledger,
    Trezor,
    Charmander,
    Exit,
}

impl MenuOptions for MainMenuOption {
    fn all() -> Vec<MainMenuOption> {
        vec![
            Self::Zecwallet,
            Self::Zcashd,
            Self::Ledger,
            Self::Trezor,
            Self::Charmander,
            Self::Exit,
        ]
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Zecwallet => "Zecwallet",
            Self::Zcashd => "zcashd",
            Self::Ledger => "Ledger",
            Self::Trezor => "Trezor",
            Self::Charmander => "Charmander",
            Self::Exit => "Exit",
        }
    }
}

pub struct MainMenu;

pub fn render(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(20), Constraint::Length(20)])
        .split(f.area());
    app.view(&Id::WelcomeComponent, f, chunks[0]);
    app.view(&Id::MainMenu, f, chunks[1]);
}

impl Mountable for MainMenu {
    fn mount(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()> {
        // Mount logo
        assert!(
            app.mount(
                Id::WelcomeComponent,
                Box::new(WelcomeComponent),
                Vec::default()
            )
            .is_ok()
        );

        // Mount main menu
        assert!(
            app.mount(
                Id::MainMenu,
                Box::new(Menu::<MainMenuOption>::new("Select the wallet to recover")),
                Vec::default(),
            )
            .is_ok()
        );
        Ok(())
    }
}

impl<T> HandleMessage<T> for MainMenu
where
    T: HasScreenAndQuit,
{
    fn handle_message(msg: Msg, model: &mut T) -> Option<Msg> {
        match msg {
            Msg::MenuSelected(option) => {
                if let Some(menu_item) = MainMenuOption::from_label(&option) {
                    match menu_item {
                        MainMenuOption::Zecwallet => model.navigate_to(Screen::ZecwalletInput),
                        MainMenuOption::Zcashd => model.navigate_to(Screen::ZcashdInput),
                        MainMenuOption::Ledger => model.navigate_to(Screen::LedgerInput),
                        MainMenuOption::Trezor
                        | MainMenuOption::Charmander
                        | MainMenuOption::Exit => model.set_quit(true),
                    }
                }
                None
            }
            _ => None,
        }
    }
}
