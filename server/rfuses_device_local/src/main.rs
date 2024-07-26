use fuser::MountOption;
use rfuse_core::sys_fs::RFuseFS;
use rfuses_device_local::init_fs::user_defined_init_fs;
use rfuses_device_local::local_fs::LocalFS;

// 使用 walkdir 和 nix 实现 local 的文件系统, 尽量避开特定操作系统
fn main() {
    env_logger::init();
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
