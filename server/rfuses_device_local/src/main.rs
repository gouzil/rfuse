use ctrlc;
use fuser::MountOption;
use log::info;
use rfuse_core::sys_fs::RFuseFS;
use rfuses_device_local::init_fs::user_defined_init_fs;
use rfuses_device_local::local_fs::LocalFS;
use rfuses_device_local::logging::init_log;

// 使用 walkdir 和 nix 实现 local 的文件系统, 尽量避开特定操作系统
fn main() {
    // 日志初始化
    init_log();
    let options = vec![
        MountOption::RW, // 这样才是读写
        // MountOption::RO, // 这样是只读
        MountOption::FSName("hello".to_string()),
        MountOption::AutoUnmount, // 这样可以自动卸载
    ];

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
            "".to_string(),
            true,
            "./sourcedata".to_string(),
            Box::new(user_defined_init_fs),
            Box::new(lfs),
        ),
        "./testdata/",
        &options,
    )
    .unwrap();

    // 等待信号，然后退出
    let () = recv.recv().unwrap();
    drop(guard)
}
