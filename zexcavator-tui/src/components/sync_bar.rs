use tui_realm_stdlib::ProgressBar;
use tuirealm::command::{Cmd, CmdResult};
use tuirealm::props::{Alignment, BorderType, Borders, Color, PropPayload, PropValue};
use tuirealm::{AttrValue, Attribute, State};
use tuirealm::{
    Component, Event, MockComponent, NoUserEvent,
    event::{Key, KeyEvent},
};

use crate::Msg;

pub struct SyncBar {
    component: ProgressBar,
}

impl Default for SyncBar {
    fn default() -> Self {
        Self {
            component: ProgressBar::default()
                .borders(
                    Borders::default()
                        .color(Color::Rgb(22, 0xC5, 0x5B))
                        .modifiers(BorderType::Rounded),
                )
                .foreground(Color::Rgb(22, 0xC5, 0x5B))
                .label("0%")
                .title("Syncing...", Alignment::Center)
                .progress(0.0),
        }
    }
}

impl MockComponent for SyncBar {
    fn view(&mut self, frame: &mut tuirealm::Frame, area: tuirealm::ratatui::prelude::Rect) {
        self.component.view(frame, area);
    }

    fn query(&self, attr: Attribute) -> Option<tuirealm::AttrValue> {
        self.component.query(attr)
    }

    fn attr(&mut self, attr: Attribute, value: tuirealm::AttrValue) {
        if attr == Attribute::Value {
            if let AttrValue::Payload(PropPayload::One(PropValue::F32(progress))) = value {
                let mut bar = std::mem::take(&mut self.component);

                // Between 0.0 and 1.0
                let fraction = (progress / 100.0).clamp(0.0, 1.0);

                bar = bar.progress(fraction.into());

                // Actual percentage
                let percentage = progress.round();
                bar = bar.label(format!("{}%", percentage));

                self.component = bar;
            }
        }
    }

    fn state(&self) -> State {
        todo!()
    }

    fn perform(&mut self, cmd: Cmd) -> CmdResult {
        self.component.perform(cmd)
    }
}

impl Component<Msg, NoUserEvent> for SyncBar {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::AppClose),
            _ => None,
        }
    }
}
