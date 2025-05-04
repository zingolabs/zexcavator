use tuirealm::{Application, Frame, NoUserEvent};

use crate::{Id, Msg};

pub mod main_menu;
pub mod zecwallet;

pub trait Mountable {
    fn mount(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()>;
}

pub trait Renderable {
    fn render(app: &mut Application<Id, Msg, NoUserEvent>, f: &mut Frame);
}
