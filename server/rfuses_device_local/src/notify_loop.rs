use log::{debug, info};
use rfuse_core::sys_fs::RFuseFSOP;

#[cfg(target_os = "linux")]
use std::path::Path;

use tokio::sync::mpsc;

use crate::ExitStatus;

#[cfg(target_os = "linux")]
pub async fn notify_loop(
    rfs_send: mpsc::Sender<RFuseFSOP>,
    source_dir: String,
) -> Result<(), ExitStatus> {
    use log::error;
    use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
    info!("[notify loop] Listening thread started");

    tokio::spawn(async move {
        loop {
            // 创建一个通道，用于接收文件改动事件
            let (notify_send, mut notify_recv) = mpsc::channel(10);

            // 创建一个文件监视器
            let mut watcher = match RecommendedWatcher::new(
                move |res| {
                    if let Err(e) = notify_send.blocking_send(res) {
                        error!("[notify loop] send error: {:?}", e);
                    }
                },
                Config::default(),
            ) {
                Ok(w) => w,
                Err(e) => {
                    error!("create watcher failed: {:?}", e);
                    break;
                }
            };

            // 监听文件夹（递归）
            match watcher.watch(Path::new(&source_dir), RecursiveMode::Recursive) {
                Ok(_) => {}
                Err(e) => {
                    error!("watcher watch failed: {:?}", e);
                    break;
                }
            };
            match notify_recv.recv().await {
                Some(event) => match event {
                    Ok(event) => match event.kind {
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                            debug!(
                                "[notify loop] event: {:?}, paths: {:?}",
                                event.kind, event.paths
                            );
                            // 检查最近哪个进程访问了该文件
                            for path in event.paths {
                                if let Some(pids) = find_pids_accessing_file(&path) {
                                    // 如果有进程访问了该文件, 那么重新初始化文件系统, 除了自己的进程
                                    if pids.len() > 1
                                        || (pids.len() == 1 && pids[0] != std::process::id() as i32)
                                    {
                                        info!("[notify loop] reinit fs");
                                        debug!("[notify loop] pids: {:?}", pids);
                                        match rfs_send.send(RFuseFSOP::ReInItFs).await {
                                            Ok(_) => debug!("[notify loop] send success"),
                                            Err(e) => {
                                                error!("[notify loop] send error: {:?}", e);
                                                break;
                                            }
                                        };
                                        break;
                                    }
                                }
                            }
                        }
                        _ => {}
                    },
                    Err(e) => {
                        error!("[notify loop] event error: {:?}", e);
                    }
                },
                None => {
                    error!("[notify loop] recv error");
                }
            }
        }
    });
    Ok(())
}

#[cfg(target_os = "linux")]
fn find_pids_accessing_file(path: &Path) -> Option<Vec<i32>> {
    use log::debug;
    use procfs::process::all_processes;
    use std::fs;

    let processes = all_processes().ok()?;
    let mut accessing_pids = Vec::new();

    // TODO: 这里有时候会出现问题, 有时候会出现找不到文件的情况, 简单来说就是处理的太快了, 导致文件已经被删除了还没来得及处理 (随着进程的增多可能会恶化这种情况)
    for process in processes.flatten() {
        let fd_dir = format!("/proc/{}/fd", process.pid);
        if let Ok(fds) = fs::read_dir(fd_dir) {
            for fd in fds.flatten() {
                if let Ok(link_path) = fs::read_link(fd.path()) {
                    if link_path == *path {
                        debug!("[find_pids_accessing_file] fd path: {:?}", fd.path());
                        accessing_pids.push(process.pid);
                    }
                }
            }
        }
    }

    if accessing_pids.is_empty() {
        None
    } else {
        Some(accessing_pids)
    }
}

#[cfg(not(target_os = "linux"))]
pub async fn notify_loop(
    rfs_send: mpsc::Sender<RFuseFSOP>,
    _source_dir: String,
) -> Result<(), ExitStatus> {
    use std::time;

    info!("[notify loop] time interval thread started");
    tokio::spawn(async move {
        loop {
            // 一分钟重制一次
            tokio::time::sleep(time::Duration::from_secs(60)).await;
            match rfs_send.send(RFuseFSOP::ReInItFs).await {
                Ok(_) => debug!("[notify loop] send success"),
                Err(e) => {
                    info!("[notify loop] send error: {:?}", e);
                    break;
                }
            };
        }
    });
    Ok(())
}
