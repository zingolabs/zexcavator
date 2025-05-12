use tui_realm_stdlib::Input;
use tuirealm::command::{Cmd, CmdResult, Direction, Position};
use tuirealm::{
    Component, Event, MockComponent, NoUserEvent,
    event::{Key, KeyEvent},
};
use tuirealm::{State, StateValue};

use crate::Msg;

#[derive(MockComponent, Default)]
pub struct BirthdayInput {
    component: Input,
}

impl BirthdayInput {
    pub fn new(initial_text: String, label: String) -> Self {
        Self {
            component: Input::default()
                .input_type(tuirealm::props::InputType::Text)
                .value(initial_text)
                .title(label, tuirealm::props::Alignment::Left),
        }
    }
}

impl Component<Msg, NoUserEvent> for BirthdayInput {
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
                return Some(Msg::BirthdayInputBlur);
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
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => self.perform(Cmd::Submit),
            _ => CmdResult::None,
        };

        match cmd {
            CmdResult::Submit(State::One(StateValue::String(_s))) => None,
            CmdResult::Changed(State::One(StateValue::String(birthday))) => {
                Some(Msg::BirthdayInputChanged(birthday))
            }
            _ => None,
        }
    }
}
