//! Main entry point for ZexCavatorCli

#![deny(warnings, missing_docs, trivial_casts, unused_qualifications)]
#![forbid(unsafe_code)]

use cli::application::APP;

/// Boot ZexCavatorCli
fn main() {
    abscissa_core::boot(&APP);
}
