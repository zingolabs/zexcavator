use std::fs;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Ok};
use chrono::Utc;
use tokio::sync::RwLock;
use tuirealm::event::{Key, KeyEvent};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::ratatui::text::Text;
use tuirealm::ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tuirealm::{Component, Frame, MockComponent, NoUserEvent, State};
use zingolib::lightclient::LightClient;

use crate::Msg;
use crate::app::model::HasScreenAndQuit;
use crate::components::HandleMessage;

#[derive(Debug, Clone)]
pub struct ExportZingolibView {
    pub light_client: Arc<RwLock<Option<LightClient>>>,
    pub saved_path: Arc<Mutex<Option<String>>>,
}

impl ExportZingolibView {
    pub fn new(light_client: Arc<RwLock<Option<LightClient>>>) -> Self {
        Self {
            light_client,
            saved_path: Arc::new(Mutex::new(None)),
        }
    }

    /// Converts the LightWallet into a Zingolib-compatible format and saves it to disk
    pub async fn do_save(&self) -> anyhow::Result<String> {
        let guard = self.light_client.read().await;
        let lc = guard.as_ref().ok_or_else(|| anyhow::anyhow!("no client"))?;

        let mut buf = Vec::new();
        {
            let mut lw_guard = lc.wallet.lock().await;
            let network = lw_guard.network;
            lw_guard
                .write(&mut buf, &network)
                .await
                .context("failed to serialize LightWallet")?;
        }

        let mut export_dir = dirs::config_dir().context("could not locate config directory")?;
        export_dir.push("zexcavator");
        export_dir.push("exports");
        export_dir.push("zingolib");

        fs::create_dir_all(&export_dir).context("failed to create export directory")?;

        let filename = format!("export-{}.dat", Utc::now().timestamp());
        let path = export_dir.join(&filename);

        fs::write(&path, &buf).context("failed to write export file")?;

        Ok(path.to_string_lossy().into_owned())
    }
}

impl MockComponent for ExportZingolibView {
    fn view(&mut self, frame: &mut Frame, area: tuirealm::ratatui::prelude::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1)])
            .split(area);

        let msg = match &*self.saved_path.lock().unwrap() {
            Some(path) => format!("Exported to:\n{}\nLoad this wallet in Zingolib.", path),
            None => "Exporting to Zingolib...".into(),
        };

        let para = Paragraph::new(Text::from(msg))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Zingolib Export"),
            )
            .wrap(Wrap { trim: false });

        frame.render_widget(para, chunks[0]);
    }

    fn query(&self, attr: tuirealm::Attribute) -> Option<tuirealm::AttrValue> {
        todo!()
    }

    fn attr(&mut self, attr: tuirealm::Attribute, value: tuirealm::AttrValue) {}

    fn state(&self) -> State {
        todo!()
    }

    fn perform(&mut self, cmd: tuirealm::command::Cmd) -> tuirealm::command::CmdResult {
        todo!()
    }
}

impl Component<Msg, NoUserEvent> for ExportZingolibView {
    fn on(&mut self, ev: tuirealm::Event<NoUserEvent>) -> Option<Msg> {
        match ev {
            tuirealm::Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => Some(Msg::Start),
            _ => None,
        }
    }
}

impl<T> HandleMessage<T> for ExportZingolibView
where
    T: HasScreenAndQuit,
{
    fn handle_message(msg: Msg, model: &mut T) -> Option<Msg> {
        None
    }
}
