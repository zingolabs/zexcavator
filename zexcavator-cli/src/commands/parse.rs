//! `parse` subcommand - example of how to write a subcommand

/// App-local prelude includes `app_reader()`/`app_writer()`/`app_config()`
/// accessors along with logging macros. Customize as you see fit.
use crate::prelude::*;

use crate::config::ZexCavatorCliConfig;
use abscissa_core::{Command, FrameworkError, Runnable, config};
use zexcavator_lib::parser::WalletParserFactory;
use zingolib::{
    config::{ChainType, ZingoConfig},
    get_latest_block_height, lightclient,
    wallet::{LightWallet, WalletBase},
};

/// `parse` subcommand
///
/// The `Parser` proc macro generates an option parser based on the struct
/// definition, and is defined in the `clap` crate. See their documentation
/// for a more comprehensive example:
///
/// <https://docs.rs/clap/>
#[derive(clap::Parser, Command, Debug)]
pub struct ParseCmd {
    /// What wallet file are we parsing?
    #[arg(required = true)]
    wallet_path: String,

    /// Enable verbose mode. A flag `-v` or `--verbose` will enable verbose mode.
    #[arg(short('v'), long("verbose"))]
    verbose: bool,
}

impl Runnable for ParseCmd {
    /// Start the application.
    fn run(&self) {
        let config = APP.config();
        println!("Config: {:#?}", config);
        let wallet_parser = WalletParserFactory::read(config.input_file.to_str().unwrap()).unwrap();

        // println!("{:#?}", wallet_parser.parser.get_wallet_name());
        wallet_parser.parser.print_internal();

        // LightClient initialization

        let seed = wallet_parser.parser.get_wallet_seed();
        let bd = wallet_parser.parser.get_birthday();

        if let Err(e) = rustls::crypto::ring::default_provider().install_default() {
            eprintln!("Error installing crypto provider: {:?}", e)
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        let mut zc = ZingoConfig::create_mainnet();
        zc.set_data_dir("wallets/".to_string());

        let config = zc.clone();

        // let latest_block_height: u32 = get_latest_block_height(uri).unwrap().try_into().unwrap();

        let initial_bh: u32 = bd.try_into().unwrap();
        let lw = LightWallet::new(
            ChainType::Mainnet,
            WalletBase::SeedBytes(seed),
            initial_bh.into(),
        )
        .unwrap();

        let mut lc = lightclient::LightClient::create_from_wallet(lw, zc, true).unwrap();
        // let latest_block =
        //     get_latest_block_height(config.lightwalletd_uri.read().unwrap().clone()).unwrap();

        rt.block_on(async {
            println!("Reading from birthday: {}", bd);
            // println!("Upto block: {}", latest_block);
            match lc.sync(true).await {
                Ok(_) => {}
                Err(e) => {
                    println!("Error syncing: {}", e);
                }
            }

            lc.await_sync().await.unwrap();
            let balances = lc.do_balance().await;

            println!("Balances: {}", balances);
        });
    }
}

impl config::Override<ZexCavatorCliConfig> for ParseCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(
        &self,
        mut config: ZexCavatorCliConfig,
    ) -> Result<ZexCavatorCliConfig, FrameworkError> {
        if !self.wallet_path.is_empty() {
            config.input_file = self.wallet_path.to_string().into();
        }

        config.verbose = self.verbose;

        Ok(config)
    }
}
