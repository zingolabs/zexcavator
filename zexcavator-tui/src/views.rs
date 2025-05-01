use tuirealm::Application;

use crate::{Id, Msg};

pub mod main_menu;
pub mod zecwallet;

pub trait Mountable {
    fn mount(app: &mut Application<Id, Msg, tuirealm::event::NoUserEvent>) -> anyhow::Result<()>;
}
