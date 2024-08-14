use log::error;
use rfuses_device_local::cli::build_cli;
use rfuses_device_local::run::run;
use rfuses_device_local::ExitStatus;
use std::process::ExitCode;

// 使用 walkdir 和 nix 实现 local 的文件系统, 尽量避开特定操作系统
fn main() -> ExitCode {
    // 解析命令行参数
    let cli = build_cli();

    match run(cli) {
        Ok(exit_code) => exit_code.into(),
        Err(e) => {
            error!("[main] run error: {:?}", e);
            ExitStatus::Error.into()
        }
    }
}
