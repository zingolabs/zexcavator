//! ZexCavatorCli Config
//!
//! See instructions in `commands.rs` to specify the path to your
//! application's configuration file and/or command-line options
//! for specifying it.

use serde::{Deserialize, Serialize};

/// ZexCavatorCli Configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ZexCavatorCliConfig {
    /// An example configuration section
    pub file: ExampleFile,
}

/// Default configuration settings.
///
/// Note: if your needs are as simple as below, you can
/// use `#[derive(Default)]` on ZexCavatorCliConfig instead.
impl Default for ZexCavatorCliConfig {
    fn default() -> Self {
        Self {
            file: ExampleFile::default(),
        }
    }
}

/// Example configuration section.
///
/// Delete this and replace it with your actual configuration structs.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExampleFile {
    /// Example configuration value
    pub wallet_file: String,
}

impl Default for ExampleFile {
    fn default() -> Self {
        Self {
            wallet_file: "world".to_owned(),
        }
    }
}
