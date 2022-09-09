//! CLI options

use std::path::PathBuf;

use argh::FromArgs;

#[derive(FromArgs)]
/// Lox interpreter
pub struct Options {
    /// an optional script to run
    #[argh(positional)]
    pub filepath: Option<PathBuf>,
}
