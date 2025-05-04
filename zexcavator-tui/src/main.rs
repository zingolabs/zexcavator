//! ## Demo
//!
//! `Demo` shows how to use tui-realm in a real case

use tuirealm::application::PollStrategy;
use tuirealm::{AttrValue, Attribute, Update};
// -- internal
mod app;
mod components;
mod views;
use app::model::Model;

// Let's define the messages handled by our app. NOTE: it must derive `PartialEq`
#[derive(Debug, PartialEq)]
pub enum Msg {
    AppClose,
    Start,
    SeedInputChanged(String),
    SeedInputValidate(String),
    MnemonicInputChanged(String),
    MnemonicInputValidate,
    MnemonicInputBlur,
    BirthdayInputChanged(String),
    BirthdayInputBlur,
    SeedInputBlur,
    MenuSelected(String),
    MenuCursorMove(usize),
    None,
}

#[derive(Debug, PartialEq)]
pub enum NavigationMsg {
    MainMenu,
    Syncing,
    ZecwalletInput,
    ZcashdInput,
}

// Let's define the component ids for our application
#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Id {
    Label,
    SeedInput,
    MnemonicInput,
    BirthdayInput,
    WelcomeComponent,
    MainMenu,
    ZecwalletView,
    ZecwalletMenu,
    ZecwalletFromPath,
    ZecwalletFromMnemonic,
    LogViewerPath,
    LogViewerSeed,
}

fn main() {
    // Setup model
    let mut model = Model::default();
    // Enter alternate screen
    let _ = model.terminal.enter_alternate_screen();
    let _ = model.terminal.enable_raw_mode();
    // Main loop
    // NOTE: loop until quit; quit is set in update if AppClose is received from counter
    while !model.quit {
        // Tick
        match model.app.tick(PollStrategy::Once) {
            Err(err) => {
                assert!(
                    model
                        .app
                        .attr(
                            &Id::Label,
                            Attribute::Text,
                            AttrValue::String(format!("Application error: {}", err)),
                        )
                        .is_ok()
                );
            }
            Ok(messages) if !messages.is_empty() => {
                // NOTE: redraw if at least one msg has been processed
                model.redraw = true;
                for msg in messages.into_iter() {
                    let mut msg = Some(msg);
                    while msg.is_some() {
                        msg = model.update(msg);
                    }
                }
            }
            Ok(_) => {
                model.redraw = true;
                model.update(None);
            }
            _ => {}
        }
        // Redraw
        if model.redraw {
            model.view();
            model.redraw = false;
        }
    }
    // Terminate terminal
    let _ = model.terminal.leave_alternate_screen();
    let _ = model.terminal.disable_raw_mode();
    let _ = model.terminal.clear_screen();
}
