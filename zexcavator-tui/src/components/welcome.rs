use tuirealm::command::CmdResult;
use tuirealm::props::{Color, Layout, Style, TextModifiers};
use tuirealm::ratatui::layout::{Constraint, Direction, Rect};
use tuirealm::ratatui::text::{Line, Span, Text};
use tuirealm::ratatui::widgets::Paragraph;
use tuirealm::{Component, Event, Frame, MockComponent, NoUserEvent, State};

use crate::Msg;

const LOGO: &str = r#"
███████╗███████╗██╗  ██╗ ██████╗ █████╗ ██╗   ██╗ █████╗ ████████╗ ██████╗ ██████╗ 
╚══███╔╝██╔════╝╚██╗██╔╝██╔════╝██╔══██╗██║   ██║██╔══██╗╚══██╔══╝██╔═══██╗██╔══██╗
  ███╔╝ █████╗   ╚███╔╝ ██║     ███████║██║   ██║███████║   ██║   ██║   ██║██████╔╝
 ███╔╝  ██╔══╝   ██╔██╗ ██║     ██╔══██║╚██╗ ██╔╝██╔══██║   ██║   ██║   ██║██╔══██╗
███████╗███████╗██╔╝ ██╗╚██████╗██║  ██║ ╚████╔╝ ██║  ██║   ██║   ╚██████╔╝██║  ██║
╚══════╝╚══════╝╚═╝  ╚═╝ ╚═════╝╚═╝  ╚═╝  ╚═══╝  ╚═╝  ██║   ╚═╝    ╚═════╝ ╚═╝  ╚═╝
                                                      ╚═╝                         
"#;

#[derive(Default)]
pub struct WelcomeComponent;

impl MockComponent for WelcomeComponent {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(2)
            .constraints(&[Constraint::Length(10)])
            .chunks(area);

        let styled_logo: Vec<Line> = LOGO
            .lines()
            .map(|l| {
                Line::from(Span::styled(
                    l,
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(TextModifiers::BOLD),
                ))
            })
            .collect();

        let logo_block =
            Paragraph::new(Text::from(styled_logo)).alignment(tuirealm::props::Alignment::Center);

        frame.render_widget(logo_block, layout[0]);
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        None
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {}

    fn state(&self) -> State {
        State::None
    }

    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        CmdResult::None
    }
}

impl Component<Msg, NoUserEvent> for WelcomeComponent {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _ = ev;
        None
    }
}
