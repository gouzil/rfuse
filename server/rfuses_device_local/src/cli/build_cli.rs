use clap::Parser;

use super::args::Cli;

pub fn build_cli() -> Cli {
    Cli::parse()
}
