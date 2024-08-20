use clap::Parser;

use super::args::Args;

pub fn build_cli() -> Args {
    Args::parse()
}
