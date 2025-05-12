use std::str::FromStr;

use bip0039::{English, Mnemonic};
use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::{
    Component, Event, MockComponent, NoUserEvent,
    event::{Key, KeyEvent},
};
use tuirealm::{State, StateValue};

use crate::Msg;

#[derive(MockComponent, Default)]
pub struct MnemonicInput {
    component: Input,
}

impl MnemonicInput {
    pub fn new(initial_text: String, label: String) -> Self {
        Self {
            component: Input::default()
                .input_type(tuirealm::props::InputType::Text)
                .value(initial_text)
                .title(label, tuirealm::props::Alignment::Left),
        }
    }

    pub fn validate_input(&self, mnemonic: String) -> bool {
        Mnemonic::<English>::from_str(&mnemonic).is_ok()
    }
}

impl Component<Msg, NoUserEvent> for MnemonicInput {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let cmd = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => {
                return Some(Msg::MnemonicInputBlur);
            } // Focus lost
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => self.perform(Cmd::Delete),
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                ..
            }) => self.perform(Cmd::Type(ch)),
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::Start),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => CmdResult::None,
            _ => CmdResult::None,
        };

        match cmd {
            CmdResult::Submit(State::One(StateValue::String(s))) => None,
            CmdResult::Changed(State::One(StateValue::String(s))) => {
                Some(Msg::MnemonicInputChanged(s))
            }
            _ => None,
        }
    }
}
