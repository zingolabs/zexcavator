use std::sync::{Arc, Mutex};

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

fn spawn_reader(command: &str, args: &[&str]) -> Arc<Mutex<Vec<String>>> {
    let output_lines = Arc::new(Mutex::new(Vec::new()));
    let shared_lines = output_lines.clone();

    let mut child = Command::new(command)
        .args(args)
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    thread::spawn(move || {
        let stdout = child.stdout.take().expect("Failed to get stdout");
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            if let Ok(line) = line {
                let mut lines = shared_lines.lock().unwrap();
                lines.push(line);
                if lines.len() > 100 {
                    lines.remove(0); // keep last 100 lines
                }
            }
        }
    });

    output_lines
}

pub type LogBuffer = Arc<Mutex<Vec<String>>>;

pub fn new_log_buffer() -> LogBuffer {
    Arc::new(Mutex::new(Vec::new()))
}

pub fn start_wallet_sync(logs: LogBuffer) {
    std::thread::spawn(move || {
        let bd = 2715348;

        if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
            logs.lock()
                .unwrap()
                .push(format!("Error installing crypto provider: {:?}", e));
        }

        let example_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut zc = ZingoConfig::create_mainnet();
        zc.set_data_dir("wallets/".to_string());

        let initial_bh: u32 = bd.try_into().unwrap();
        let lw = LightWallet::new(
            ChainType::Mainnet,
            WalletBase::MnemonicPhrase(example_phrase.to_owned()),
            initial_bh.into(),
        )
        .unwrap();

        let mut lc = lightclient::LightClient::create_from_wallet(lw, zc, true).unwrap();

        rt.block_on(async {
            logs.lock()
                .unwrap()
                .push(format!("Starting sync from birthday: {}", bd));
            match lc.sync(true).await {
                Ok(_) => logs.lock().unwrap().push("Sync started".to_string()),
                Err(e) => logs
                    .lock()
                    .unwrap()
                    .push(format!("Error starting syncing: {}", e)),
            }

            loop {
                sleep(Duration::from_secs(1)).await;
                match lc.poll_sync() {
                    PollReport::NoHandle => break,
                    PollReport::NotReady => (),
                    PollReport::Ready(result) => match result {
                        Ok(sync_result) => {
                            logs.lock()
                                .unwrap()
                                .push(format!("Sync result: {}", sync_result));
                            break;
                        }
                        Err(e) => {
                            logs.lock().unwrap().push(format!("{}", e));
                            logs.lock().unwrap().push("Restarting sync".to_string());
                            match lc.sync(true).await {
                                Ok(_) => logs.lock().unwrap().push("Sync restarted".to_string()),
                                Err(e) => logs
                                    .lock()
                                    .unwrap()
                                    .push(format!("Error restarting syncing: {}", e)),
                            }
                            continue;
                        }
                    },
                };
            }

            match lc.await_sync().await {
                Ok(_) => logs.lock().unwrap().push("Sync finished".to_string()),
                Err(e) => logs.lock().unwrap().push(format!("{}", e)),
            }
            let balances = lc.do_balance().await;
            logs.lock()
                .unwrap()
                .push(format!("Balances: {:?}", balances));
        });
    });
}

pub struct LogViewer {
    logs: LogBuffer,
}

impl LogViewer {
    pub fn new(logs: LogBuffer) -> Self {
        Self { logs }
    }
}

impl Default for LogViewer {
    fn default() -> Self {
        Self::new(new_log_buffer())
    }
}

use tokio::time::sleep;
use tuirealm::ratatui::layout::Rect;
use tuirealm::{Component, Event, Frame, MockComponent, NoUserEvent};
use zingolib::config::{ChainType, ZingoConfig};
use zingolib::data::PollReport;
use zingolib::lightclient;
use zingolib::wallet::{LightWallet, WalletBase};

use crate::Msg;

impl MockComponent for LogViewer {
    fn view(&mut self, frame: &mut Frame, area: Rect) {
        use tuirealm::ratatui::text::{Line, Span, Text};
        use tuirealm::ratatui::widgets::{Block, Borders, Paragraph};

        let log_lines = self.logs.lock().unwrap();
        let text = Text::from(
            log_lines
                .iter()
                .map(|l| Line::from(Span::raw(l)))
                .collect::<Vec<_>>(),
        );

        let paragraph =
            Paragraph::new(text).block(Block::default().title("Sync Log").borders(Borders::ALL));

        frame.render_widget(paragraph, area);
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        todo!()
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {
        todo!()
    }

    fn state(&self) -> tuirealm::State {
        todo!()
    }

    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        todo!()
    }

    // Other trait methods (attr, state, etc.) as no-ops
}

impl Component<Msg, NoUserEvent> for LogViewer {
    fn on(&mut self, _ev: Event<NoUserEvent>) -> Option<Msg> {
        None // Nothing needed — Tick drives redraw
    }
}
