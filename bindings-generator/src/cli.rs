//! CLI arguments handling.
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    Single(Single),
    History(History),
}

#[derive(Args)]
pub struct Single {
    #[arg(short, long)]
    pub input_header: PathBuf,

    #[arg(short, long, default_value = "bindings-generator/input/whitelist.txt")]
    pub whitelist: PathBuf,

    #[arg(
        short,
        long,
        default_value = "amd-smi-wrapper-sys/src/versions/latest.rs"
    )]
    pub output: PathBuf,
}

/// Generate a base binding and multiple partial bindings, based on a history analysed with `ch-hist`.
///
///  This assumes that you have already run `ch-hist`, for instance with:
///
/// ```
/// ch-hist -i all-amdsmi-versions-to-support -o report-1/ --whitelist ./whitelist.txt --generate-code new
/// ```
#[derive(Args)]
pub struct History {
    ///
    #[arg(short, long)]
    pub input_report: PathBuf,

    #[arg(short, long, default_value = "amd-smi-wrapper-sys/src/versions")]
    pub output_dir: PathBuf,
}
