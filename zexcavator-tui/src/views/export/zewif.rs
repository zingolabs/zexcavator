use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::{Context, Ok};
use bc_envelope::Envelope;
use bc_envelope::prelude::CBOREncodable;
use chrono::Utc;
use tokio::sync::RwLock;
use tuirealm::event::{Key, KeyEvent, KeyModifiers};
use tuirealm::ratatui::layout::{Constraint, Direction, Layout};
use tuirealm::ratatui::text::Text;
use tuirealm::ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tuirealm::{Component, Frame, MockComponent, NoUserEvent, State};
use zewif::{Bip39Mnemonic, BlockHeight, SeedMaterial, Zewif, ZewifWallet};
use zingolib::grpc_connector::get_latest_block;
use zingolib::lightclient::LightClient;

use crate::Msg;
use crate::app::model::HasScreenAndQuit;
use crate::components::HandleMessage;

#[derive(Debug, Clone)]
pub struct ExportZewifView {
    pub light_client: Arc<RwLock<Option<LightClient>>>,
    pub saved_path: Arc<Mutex<Option<String>>>,
}

impl ExportZewifView {
    pub fn new(light_client: Arc<RwLock<Option<LightClient>>>) -> Self {
        Self {
            light_client,
            saved_path: Arc::new(Mutex::new(None)),
        }
    }

    /// Converts the LightWallet into a ZeWIF-compatible format and saves it to disk
    pub async fn do_save(&self) -> anyhow::Result<String> {
        let guard = self.light_client.read().await;
        let lc = guard.as_ref().ok_or_else(|| anyhow::anyhow!("no client"))?;

        let wallet_guard = lc.wallet.lock().await;
        let mnemonic = wallet_guard.mnemonic().cloned();
        drop(wallet_guard);

        let export_height = get_latest_block(lc.get_server_uri()).await.unwrap().height as u32;
        drop(guard);

        let (m, _) = mnemonic.unwrap();

        let path = ExportZewifView::export_to_zewif(Some(m), export_height);

        Ok(path.unwrap().to_string_lossy().to_string())
    }

    /// Inline implementation of zingolib's LichClient to ZeWIF conversion.
    /// Eventually, this will be moved to the `zewif-zingolib` crate.
    pub fn export_to_zewif(
        mnemonic: Option<bip0039::Mnemonic>,
        export_height: u32,
    ) -> Result<PathBuf, anyhow::Error> {
        let seed_material: Option<SeedMaterial> = match mnemonic {
            Some(m) => {
                let phrase = m.clone().into_phrase();

                let zewif_bip39_mnemonic =
                    Bip39Mnemonic::new(phrase, Some(zewif::MnemonicLanguage::English));
                Some(SeedMaterial::Bip39Mnemonic(zewif_bip39_mnemonic))
            }
            None => None,
        };

        let mut zewif_wallet: ZewifWallet = ZewifWallet::new(zewif::Network::Main);

        match seed_material {
            Some(seed_material) => zewif_wallet.set_seed_material(seed_material),
            None => panic!("no seed material"),
        }

        let zewif: Zewif = Zewif::new(BlockHeight::from_u32(export_height));

        let mut export_dir = dirs::config_dir().context("could not locate config directory")?;
        export_dir.push("zexcavator");
        export_dir.push("exports");

        fs::create_dir_all(&export_dir)
            .with_context(|| format!("failed to create directory {:?}", export_dir))?;

        // Save to path zexcavator-<timestamp>.zewif
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();

        let filename = format!("zexcavator-{}.zewif", timestamp);
        let path = export_dir.join(filename);

        // Convert the Zewif instance to an Envelope
        let envelope = Envelope::from(zewif.clone());
        std::fs::write(&path, envelope.to_cbor_data()).unwrap();
        Ok(path)
    }
}

impl MockComponent for ExportZewifView {
    fn view(&mut self, frame: &mut Frame, area: tuirealm::ratatui::prelude::Rect) {
        // Split off a little header row if you like, or just draw full-screen:
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(1)])
            .split(area);

        let msg = match &*self.saved_path.lock().unwrap() {
            Some(path) => format!("Exported to:\n{}", path),
            None => "Exporting ZeWIF…".into(),
        };

        let para = Paragraph::new(Text::from(msg))
            .block(Block::default().borders(Borders::ALL).title("ZeWIF Export"))
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

impl Component<Msg, NoUserEvent> for ExportZewifView {
    fn on(&mut self, ev: tuirealm::Event<NoUserEvent>) -> Option<Msg> {
        // if let Some(menu_msg) = self.menu.on(ev.clone()) {
        //     return Some(menu_msg);
        // }
        match ev {
            tuirealm::Event::Keyboard(KeyEvent {
                code: Key::Esc,
                modifiers: KeyModifiers::NONE,
            }) => Some(Msg::Start),
            _ => None,
        }
    }
}

impl<T> HandleMessage<T> for ExportZewifView
where
    T: HasScreenAndQuit,
{
    fn handle_message(msg: Msg, model: &mut T) -> Option<Msg> {
        None
    }
}
