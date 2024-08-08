use std::path::PathBuf;

use clap::{command, Parser};

use crate::logging::LogLevel;

#[derive(Parser)]
#[command(version, author, about, long_about = None)]
pub struct Cli {
    #[clap(help = "Origin file address [default: .]")]
    pub origin: PathBuf,

    #[clap(help = "Mount point address [default: .]")]
    pub mount: PathBuf,

    #[clap(short, long, help = "Read only")]
    pub read_only: bool,

    #[clap(default_value = "rfuses", help = "Set the name of the source in mtab.")]
    pub fs_name: String,

    #[clap(flatten)]
    pub log_level_args: LogLevelArgs,
}

#[derive(Debug, clap::Args)]
pub struct LogLevelArgs {
    /// Enable verbose logging.
    #[arg(
        short,
        long,
        global = true,
        group = "verbosity",
        help_heading = "Log levels"
    )]
    pub verbose: bool,
    /// Print diagnostics, but nothing else.
    #[arg(
        short,
        long,
        global = true,
        group = "verbosity",
        help_heading = "Log levels"
    )]
    pub quiet: bool,
    /// Disable all logging (but still exit with status code "1" upon detecting diagnostics).
    #[arg(
        short,
        long,
        global = true,
        group = "verbosity",
        help_heading = "Log levels"
    )]
    pub silent: bool,
}

impl From<&LogLevelArgs> for LogLevel {
    fn from(args: &LogLevelArgs) -> Self {
        if args.silent {
            Self::Silent
        } else if args.quiet {
            Self::Quiet
        } else if args.verbose {
            Self::Verbose
        } else {
            Self::Default
        }
    }
}
