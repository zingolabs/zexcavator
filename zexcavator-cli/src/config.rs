//! ZexCavatorCli Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const DEFAULT_OUTPUT_FILE: &str = "export.zewif";

/// ZexCavatorCli Configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ZexCavatorCliConfig {
    /// Input file where to read from
    pub input_file: PathBuf,

    /// Output file. Defaults to `export.zewif`
    pub output_file: PathBuf,

    /// verbose mode
    pub verbose: bool,
}

/// Default configuration settings.
impl Default for ZexCavatorCliConfig {
    fn default() -> Self {
        Self {
            input_file: String::from("").into(),
            output_file: String::from(DEFAULT_OUTPUT_FILE).into(),
            verbose: false,
        }
    }
}
