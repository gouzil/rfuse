use std::process::ExitCode;

use fuser::MountOption;
use log::info;
use rfuse_core::sys_fs::RFuseFS;
use rfuses_device_local::cli::build_cli;
use rfuses_device_local::init_fs::user_defined_init_fs;
use rfuses_device_local::local_fs::LocalFS;
use rfuses_device_local::logging::{init_log, LogLevel};
use rfuses_device_local::ExitStatus;

// 使用 walkdir 和 nix 实现 local 的文件系统, 尽量避开特定操作系统
fn main() -> ExitCode {
    // 解析命令行参数
    let cli = build_cli();

    // 日志初始化
    let log_level = LogLevel::from(&cli.log_level_args);
    init_log(&log_level);

    // 挂载选项
    let mut options = vec![
        MountOption::FSName(cli.fs_name.clone()), // 文件系统名称
        MountOption::AutoUnmount,                 // 这样可以自动卸载
    ];

    if cli.read_only {
        options.push(MountOption::RO); // 这样是只读
    } else {
        options.push(MountOption::RW); // 这样才是读写
    }

    // 创建信号处理器
    let (send, recv) = std::sync::mpsc::channel();
    // 这里是 ctrl+c 信号处理
    ctrlc::set_handler(move || {
        info!("ctrl+c handler running");
        send.send(()).unwrap();
    })
    .unwrap();
    // 创建本地文件系统
    let lfs = LocalFS;
    let guard = fuser::spawn_mount2(
        RFuseFS::new(
            cli.fs_name,
            true,
            cli.origin.display().to_string(),
            Box::new(user_defined_init_fs),
            Box::new(lfs),
        ),
        cli.mount.display().to_string(),
        &options,
    )
    .unwrap();

    // 等待信号，然后退出
    let () = recv.recv().unwrap();
    drop(guard);

    ExitStatus::Success.into()
}
