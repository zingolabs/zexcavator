//! ## Demo
//!
//! `Demo` shows how to use tui-realm in a real case

use std::io;

use components::log_viewer::SyncSource;
use tuirealm::application::PollStrategy;
use tuirealm::ratatui::crossterm::event::DisableMouseCapture;
use tuirealm::ratatui::crossterm::execute;
use tuirealm::ratatui::crossterm::terminal::{LeaveAlternateScreen, disable_raw_mode};
use tuirealm::{AttrValue, Attribute, Update};
mod app;
mod components;
mod views;
use app::model::Model;
use zingolib::lightclient::PoolBalances;

#[derive(Debug, PartialEq)]
pub enum Msg {
    AppClose,
    Start,
    SeedInputChanged(String),
    SeedInputValidate(String),
    MnemonicInputChanged(String),
    MnemonicInputBlur,
    StartSync(SyncSource),
    BirthdayInputChanged(String),
    BirthdayInputBlur,
    FromPathSubmitBlur,
    FromMnemonicSubmitBlur,
    FromPathInputBlur,
    MenuSelected(String),
    MenuCursorMove(usize),
    FromMnemonicSubmit,
    FromPathSubmit,
    GoToResult,
    InitializeLightClient,
    BalanceReady(PoolBalances),
    FetchBalance,
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
    MnemonicInput,
    BirthdayInput,
    WelcomeComponent,
    MainMenu,
    ZecwalletView,
    ZecwalletMenu,
    ZecwalletFromPath,
    ZecwalletFromMnemonic,
    ZecwalletFromPathButton,
    ZecwalletFromMnemonicButton,
    SyncLog,
    ProgressBar,
    ExportView,
    ExportMenu,
    ResultViewer,
    ExportZewif,
    ExportSend,
    ExportZingolib,
}

#[tokio::main]
async fn main() {
    // Setup model
    let mut model = Model::default();
    std::panic::set_hook(Box::new(|info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        // Show WHY we crashed
        eprintln!("{}", info);
    }));

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
                    let mut current = Some(msg);
                    while let Some(next) = current {
                        current = model.update(Some(next));
                    }
                }
            }
            Ok(_) => {
                model.redraw = true;
                model.update(None);
            }
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
