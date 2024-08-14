use std::sync::mpsc;

use crate::init_fs::user_defined_init_fs;
use crate::notify_loop::notify_loop;
use crate::{
    cli::args::Cli,
    local_fs::LocalFS,
    logging::{init_log, LogLevel},
    ExitStatus,
};
use anyhow::Result;
use fuser::MountOption;
use log::{error, info};
use rfuse_core::sys_fs::{RFuseFS, RFuseFSOP};

pub fn run(
    Cli {
        origin,
        mount,
        read_only,
        fs_name,
        log_level_args,
    }: Cli,
) -> Result<ExitStatus> {
    // 日志初始化
    let log_level = LogLevel::from(&log_level_args);
    init_log(&log_level);

    // 挂载选项
    let mut options = vec![
        MountOption::FSName(fs_name.clone()), // 文件系统名称
        MountOption::AutoUnmount,             // 这样可以自动卸载
        MountOption::AllowOther,
    ];

    if read_only {
        options.push(MountOption::RO); // 这样是只读
    } else {
        options.push(MountOption::RW); // 这样才是读写
    }

    // 创建信号处理器
    let (rfs_send, rfs_recv) = mpsc::channel();

    let ctrlc_send = rfs_send.clone();
    // 这里是 ctrl+c 信号处理
    ctrlc::set_handler(move || {
        info!("ctrl+c handler running");
        ctrlc_send.send(RFuseFSOP::Exit).unwrap();
    })?;

    if let Err(e) = notify_loop(rfs_send.clone(), origin.display().to_string()) {
        error!("[run] notify_loop failed");
        return Ok(e);
    }

    loop {
        // 创建本地文件系统
        let lfs = LocalFS;
        let rfs = RFuseFS::new(
            fs_name.clone(),
            true,
            origin.display().to_string(),
            Box::new(user_defined_init_fs),
            Box::new(lfs),
        );
        let guard = fuser::spawn_mount2(rfs, mount.display().to_string(), &options).unwrap();

        if let Ok(rfs_op) = rfs_recv.recv() {
            match rfs_op {
                RFuseFSOP::ReInItFs => {
                    drop(guard);
                    continue;
                }
                RFuseFSOP::Exit => {
                    drop(guard);
                    break;
                }
                RFuseFSOP::Nothing => {
                    // do nothing
                    // Note: 或许不应该有这个参数，这样会导致 guard 不断的被加载
                }
            }
        }
    }

    Ok(ExitStatus::Success)
}
