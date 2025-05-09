//! ## Components
//!
//! demo example components

use tuirealm::props::{Alignment, Borders, Color, Style};
use tuirealm::ratatui::widgets::Block;

use super::Msg;

// -- modules
pub mod birthday_input;
pub mod input;
pub mod label;
pub mod log_viewer;
pub mod menu;
pub mod mnemonic_input;
pub mod result_viewer;
pub mod sync_bar;
pub mod welcome;

/// ### get_block
///
/// Get block
pub(crate) fn get_block<'a>(props: Borders, title: (String, Alignment), focus: bool) -> Block<'a> {
    Block::default()
        .borders(props.sides)
        .border_style(match focus {
            true => props.style(),
            false => Style::default().fg(Color::Reset).bg(Color::Reset),
        })
        .border_type(props.modifiers)
        .title(title.0)
        .title_alignment(title.1)
}

pub trait HandleMessage<T> {
    fn handle_message(msg: Msg, model: &mut T) -> Option<Msg>;
}
