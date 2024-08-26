use colored::Colorize;
use fern;
use log::Level;
use std::path::Path;

pub fn init_log(level: &LogLevel) {
    let mut logger = fern::Dispatch::new()
        .format(|out, message, record| match record.level() {
            Level::Error => {
                out.finish(format_args!(
                    "[{}:{}][{}] {}",
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    "ERROR".red(),
                    message
                ));
            }
            Level::Warn => {
                out.finish(format_args!(
                    "[{}:{}][{}] {}",
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    "WARNING".yellow(),
                    message
                ));
            }
            Level::Info | Level::Debug | Level::Trace => {
                out.finish(format_args!(
                    "{}[{}][{}:{}][{}] {}",
                    chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                    record.target(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    match record.level() {
                        Level::Info => "INFO".green(),
                        Level::Debug => "DEBUG".blue(),
                        Level::Trace => "Trace".purple(),
                        Level::Error => "Error".red(),
                        Level::Warn => "Warn".yellow(),
                    },
                    message
                ));
            }
        })
        .level(level.level_filter());
    // .level_for("fuser", log::LevelFilter::Warn);

    // debug 模式下输出到 stderr
    if cfg!(debug_assertions) {
        logger = logger.chain(std::io::stdout());
    } else {
        // 查看文件夹是否存在
        let log_dir_path = Path::new("./logs/");
        if !log_dir_path.is_dir() {
            std::fs::create_dir(log_dir_path).unwrap();
        }
        let log_dir = fern::DateBased::new(log_dir_path, "%Y-%m-%d-rfuses.log");
        logger = logger.chain(log_dir);
    }

    logger.apply().unwrap();
}

#[derive(Debug, Default, PartialOrd, Ord, PartialEq, Eq, Copy, Clone)]
pub enum LogLevel {
    /// No output ([`log::LevelFilter::Off`]).
    Silent,
    /// Only show lint violations, with no decorative output
    /// ([`log::LevelFilter::Off`]).
    Quiet,
    /// All user-facing output ([`log::LevelFilter::Info`]).
    #[default]
    Default,
    /// All user-facing output ([`log::LevelFilter::Debug`]).
    Verbose,
}

impl LogLevel {
    #[allow(clippy::trivially_copy_pass_by_ref)]
    const fn level_filter(&self) -> log::LevelFilter {
        match self {
            LogLevel::Default => log::LevelFilter::Info,
            LogLevel::Verbose => log::LevelFilter::Debug,
            LogLevel::Quiet => log::LevelFilter::Off,
            LogLevel::Silent => log::LevelFilter::Off,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::logging::LogLevel;

    #[test]
    fn ordering() {
        assert!(LogLevel::Default > LogLevel::Silent);
        assert!(LogLevel::Default >= LogLevel::Default);
        assert!(LogLevel::Quiet > LogLevel::Silent);
        assert!(LogLevel::Verbose > LogLevel::Default);
        assert!(LogLevel::Verbose > LogLevel::Silent);
    }
}
