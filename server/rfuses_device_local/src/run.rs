use crate::init_fs::user_defined_init_fs;
use crate::notify_loop::notify_loop;
use crate::{
    cli::args::{Args, Command, LinkCommand},
    local_fs::LocalFS,
    logging::{init_log, LogLevel},
    ExitStatus,
};
use anyhow::Result;
use fuser::MountOption;
use log::{debug, error, info};
use nix::unistd::geteuid;
use rfuse_core::sys_fs::{RFuseFS, RFuseFSOP};
use rfuse_device_disk::DiskType;
use tokio::{signal, sync::mpsc};

pub async fn run(
    Args {
        command,
        log_level_args,
    }: Args,
) -> Result<ExitStatus> {
    // 日志初始化
    let log_level = LogLevel::from(&log_level_args);
    init_log(&log_level);

    match command {
        Command::Link(link_command) => run_link(link_command).await,
    }
}

async fn run_link(
    LinkCommand {
        origin,
        mount,
        read_only,
        fs_name,
        disk_type,
    }: LinkCommand,
) -> Result<ExitStatus> {
    // 挂载选项
    let mut options = vec![
        MountOption::FSName(fs_name.clone()), // 文件系统名称
                                              // MountOption::AutoUnmount,             // 这样可以自动卸载, (但是会导致loop情况下重复卸载)
    ];

    if geteuid().is_root() {
        options.push(MountOption::AllowOther); // 这样可以让其他用户访问
    }

    if read_only {
        options.push(MountOption::RO); // 这样是只读
    } else {
        options.push(MountOption::RW); // 这样才是读写
    }

    // 创建信号处理器
    let (rfs_send, mut rfs_recv) = mpsc::channel(3);

    let ctrlc_send = rfs_send.clone();
    // 这里是 ctrl+c 信号处理
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(_) => debug!("ctrl+c received"),
            Err(e) => error!("ctrl+c error: {:?}", e),
        };
        info!("ctrl+c handler running");
        match ctrlc_send.send(RFuseFSOP::Exit).await {
            Ok(_) => debug!("ctrl+c handler send success"),
            Err(e) => error!("ctrl+c handler send error: {:?}", e),
        }
    });

    if let Err(e) = notify_loop(rfs_send.clone(), origin.display().to_string()).await {
        error!("[run] notify_loop failed");
        return Ok(e);
    }

    loop {
        // 创建本地文件系统
        let lfs = match DiskType::from(&disk_type) {
            DiskType::Local => LocalFS,
            DiskType::Mem => unimplemented!(),
        };
        let rfs = RFuseFS::new(
            fs_name.clone(),
            true,
            origin.display().to_string(),
            Box::new(user_defined_init_fs),
            Box::new(lfs),
        );
        let guard = fuser::spawn_mount2(rfs, mount.display().to_string(), &options).unwrap();

        if let Some(rfs_op) = rfs_recv.recv().await {
            match rfs_op {
                RFuseFSOP::ReInItFs => {
                    drop(guard);
                    continue;
                }
                RFuseFSOP::Exit => {
                    guard.join();
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
