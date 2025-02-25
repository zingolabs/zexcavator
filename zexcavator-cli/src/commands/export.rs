//! `parse` subcommand - example of how to write a subcommand

use std::{f32::consts::E, path::PathBuf, str::FromStr};

/// App-local prelude includes `app_reader()`/`app_writer()`/`app_config()`
/// accessors along with logging macros. Customize as you see fit.
use crate::prelude::*;

use crate::config::ZexCavatorCliConfig;
use abscissa_core::{config, Command, FrameworkError, Runnable};
use zexcavator::parser::WalletParserFactory;

/// `parse` subcommand
///
/// The `Parser` proc macro generates an option parser based on the struct
/// definition, and is defined in the `clap` crate. See their documentation
/// for a more comprehensive example:
///
/// <https://docs.rs/clap/>
#[derive(clap::Parser, Command, Debug)]
pub struct ExportCmd {
    /// A wallet file. Currently only ZecWallet and YWallet are supported.
    #[arg(required = true, value_name = "INPUT_FILE")]
    input_file: String,

    /// Where to save the ZeWIF file.
    #[arg(value_name = "OUTPUT_FILE")]
    output_file: Option<String>,
}

impl Runnable for ExportCmd {
    /// Start the application.
    fn run(&self) {
        unimplemented!();
    }
}

impl config::Override<ZexCavatorCliConfig> for ExportCmd {
    // Process the given command line options, overriding settings from
    // a configuration file using explicit flags taken from command-line
    // arguments.
    fn override_config(
        &self,
        mut config: ZexCavatorCliConfig,
    ) -> Result<ZexCavatorCliConfig, FrameworkError> {
        config.input_file = PathBuf::from_str(&self.input_file).unwrap();

        if let Some(output_file) = &self.output_file {
            config.output_file = PathBuf::from_str(output_file).unwrap();
        }

        Ok(config)
    }
}
