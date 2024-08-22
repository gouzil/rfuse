use log::error;
use rfuses_device_local::cli::build_cli;
use rfuses_device_local::run::run;
use rfuses_device_local::ExitStatus;
use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    // 解析命令行参数
    let cli = build_cli();

    match run(cli).await {
        Ok(exit_code) => exit_code.into(),
        Err(e) => {
            error!("[main] run error: {:?}", e);
            ExitStatus::Error.into()
        }
    }
}
