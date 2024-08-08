use std::process::ExitCode;

pub mod cli;
pub mod init_fs;
pub mod local_fs;
pub mod logging;
pub mod utils;

#[derive(Copy, Clone)]
pub enum ExitStatus {
    /// 成功
    Success,
    /// cli 参数解析错误，或者其他错误
    Failure,
    /// rfuse 内部错误
    Error,
}

impl From<ExitStatus> for ExitCode {
    fn from(status: ExitStatus) -> Self {
        match status {
            ExitStatus::Success => ExitCode::from(0),
            ExitStatus::Failure => ExitCode::from(1),
            ExitStatus::Error => ExitCode::from(2),
        }
    }
}
