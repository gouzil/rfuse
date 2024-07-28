use fuser::MountOption;
use rfuse_core::sys_fs::RFuseFS;
use rfuses_device_local::init_fs::user_defined_init_fs;
use rfuses_device_local::local_fs::LocalFS;

// 日志初始化
#[cfg(debug_assertions)]
fn init_log() {
    env_logger::init();
}

#[cfg(not(debug_assertions))]
fn init_log() {
    use flexi_logger::{Cleanup, Criterion, FileSpec, Logger, Naming, WriteMode};
    Logger::try_with_str("info")
        .unwrap()
        .log_to_file(FileSpec::default().directory("./logs").suppress_timestamp()) // 日志文件目录
        .write_mode(WriteMode::BufferAndFlush) // 有缓存的写入模式
        // .write_mode(WriteMode::Direct) // 直接写入模式
        .rotate(
            Criterion::Size(10 * 1024 * 1024), // 每个文件 10M
            Naming::Numbers,
            Cleanup::KeepLogFiles(7),
        )
        .start()
        .unwrap();
}

// 使用 walkdir 和 nix 实现 local 的文件系统, 尽量避开特定操作系统
fn main() {
    init_log();
    let options = vec![
        MountOption::RW, // 这样才是读写
        // MountOption::RO, // 这样是只读
        MountOption::FSName("hello".to_string()),
        MountOption::AutoUnmount, // 这样可以自动卸载
    ];
    let lfs = LocalFS;
    fuser::mount2(
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
    .unwrap()
}
