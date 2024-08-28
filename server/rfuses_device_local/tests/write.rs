use std::{
    fs::{self, File},
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    path::Path,
};

use common::{rfuses_spawn_run, run_command_with_status, TestContext};

mod common;

#[tokio::test]
async fn test_write_file() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    let closure = || {
        // 测试文件写入是否成功
        let test_file = "test_write.txt";
        // 写入文件到挂载目录
        let mut test_file_mount = mount_path.clone();
        test_file_mount.push(test_file);
        // 写入文件内容
        let content = "Hello, World!";
        let mut file = File::create(&test_file_mount).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        // 检查文件是否存在于原始目录
        let mut test_file_origin = origin_path.clone();
        test_file_origin.push(test_file);
        assert!(Path::new(&test_file_origin).exists());
        // 读取文件内容
        let mut file = File::open(&test_file_origin).unwrap();
        let mut read_content = String::new();
        file.read_to_string(&mut read_content).unwrap();
        // 检查文件内容是否相同
        assert_eq!(content, read_content);

        // 测试meta信息写入是否成功
        let test_file_mount_meta = fs::metadata(test_file_mount).unwrap();
        let test_file_origin_meta = fs::metadata(test_file_origin).unwrap();

        // 检查文件大小是否相同
        assert_eq!(test_file_mount_meta.size(), test_file_origin_meta.size());
        // 检查文件权限是否相同
        assert_eq!(test_file_mount_meta.mode(), test_file_origin_meta.mode());
        // 检查文件所有者是否相同
        assert_eq!(test_file_mount_meta.uid(), test_file_origin_meta.uid());
        // 检查文件所属组是否相同
        assert_eq!(test_file_mount_meta.gid(), test_file_origin_meta.gid());
        // 检查文件修改时间是否相同
        assert_eq!(test_file_mount_meta.mtime(), test_file_origin_meta.mtime());
        // 检查文件访问时间是否相同
        assert_eq!(test_file_mount_meta.atime(), test_file_origin_meta.atime());
        // 检查文件创建时间是否相同
        assert_eq!(test_file_mount_meta.ctime(), test_file_origin_meta.ctime());
        // 检查文件类型是否相同
        assert_eq!(
            test_file_mount_meta.file_type(),
            test_file_origin_meta.file_type()
        );
        // 检查inode号是否相同
        assert_eq!(test_file_mount_meta.ino(), test_file_origin_meta.ino());
    };
    rfuses_spawn_run!(
        {
            context
                .link()
                .arg(context.origin_dir.path())
                .arg(context.mount_dir.path())
        },
        closure
    );
}
