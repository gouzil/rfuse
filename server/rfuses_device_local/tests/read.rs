use std::{
    fs::{self, File},
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    path::Path,
};

use common::{rfuses_spawn_run, run_command_with_status, TestContext};
use rand::Rng;

mod common;

#[tokio::test]
async fn test_read_file() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    // 测试文件读取是否成功
    let test_file = "test_read.txt";
    // 写入文件到原始目录
    let mut test_file_origin = origin_path.clone();
    test_file_origin.push(test_file);
    // 写入文件内容
    let content = "Hello, World!";
    let mut file = File::create(&test_file_origin).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    // 检查文件是否存在于原始目录
    assert!(Path::new(&test_file_origin).exists());

    let closure = || {
        // 读取挂载文件内容
        let mut test_file_mount = mount_path.clone();
        test_file_mount.push(test_file);
        let mut file = File::open(&test_file_mount).unwrap();
        let mut read_content = String::new();
        file.read_to_string(&mut read_content).unwrap();
        // 检查文件内容是否相同
        assert_eq!(content, read_content);

        // 测试meta信息读取是否成功
        let test_file_origin_meta = fs::metadata(&test_file_origin).unwrap();
        let test_file_mount_meta = fs::metadata(&test_file_mount).unwrap();

        assert_ne!(test_file_origin, test_file_mount);

        // 检查inode号是否相同
        assert_eq!(test_file_origin_meta.ino(), test_file_mount_meta.ino());
        // 检查文件大小是否相同
        assert_eq!(test_file_origin_meta.size(), test_file_mount_meta.size());
        // 检查文件权限是否相同
        assert_eq!(test_file_origin_meta.mode(), test_file_mount_meta.mode());
        // 检查文件所有者是否相同
        assert_eq!(test_file_origin_meta.uid(), test_file_mount_meta.uid());
        // 检查文件所属组是否相同
        assert_eq!(test_file_origin_meta.gid(), test_file_mount_meta.gid());
        // 检查文件修改时间是否相同
        assert_eq!(test_file_origin_meta.mtime(), test_file_mount_meta.mtime());
        // 检查文件访问时间是否相同
        // assert_eq!(test_file_origin_meta.atime(), test_file_mount_meta.atime());
        // 检查文件创建时间是否相同
        assert_eq!(test_file_origin_meta.ctime(), test_file_mount_meta.ctime());
        // 检查文件类型是否相同
        assert_eq!(
            test_file_origin_meta.file_type(),
            test_file_mount_meta.file_type()
        );
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

#[tokio::test]
async fn test_read_exact_file_continuous() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    let test_file = "test_read_exact_file_continuous.txt";
    let mut test_file_origin = origin_path.clone();
    test_file_origin.push(test_file);
    const FILE_SIZE: usize = 1024 * 1024 * 10; // 10M
    let mut content = vec![0u8; FILE_SIZE];
    rand::thread_rng().fill(&mut content[..]);

    let mut file = File::create(&test_file_origin).unwrap();
    file.write_all(&content).unwrap();
    assert!(Path::new(&test_file_origin).exists());

    let closure = || {
        let mut test_file_mount = mount_path.clone();
        test_file_mount.push(test_file);
        let mut buf = vec![0u8; FILE_SIZE];
        let mut f = fs::OpenOptions::new()
            .read(true)
            .open(&test_file_mount)
            .unwrap();
        f.read_exact(&mut buf).unwrap();
        assert_eq!(content, &buf[..]);
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

#[tokio::test]
async fn test_read_sub_dir() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    // 写入子目录到原始目录
    let test_dir = "test_read_dir/";
    let mut test_dir_origin = origin_path.clone();
    test_dir_origin.push(test_dir);
    fs::create_dir(&test_dir_origin).unwrap();
    // 检查子目录是否存在于原始目录
    assert!(Path::new(&test_dir_origin).exists());

    let closure = || {
        // 读取挂载子目录
        let mut test_dir_mount = mount_path.clone();
        test_dir_mount.push(test_dir);
        assert!(Path::new(&test_dir_mount).exists());

        // 测试meta信息读取是否成功
        let test_dir_origin_meta = fs::metadata(&test_dir_origin).unwrap();
        let test_dir_mount_meta = fs::metadata(&test_dir_mount).unwrap();

        assert_ne!(test_dir_origin, test_dir_mount);

        // 检查inode号是否相同
        assert_eq!(test_dir_origin_meta.ino(), test_dir_mount_meta.ino());
        // 检查文件权限是否相同
        assert_eq!(test_dir_origin_meta.mode(), test_dir_mount_meta.mode());
        // 检查文件所有者是否相同
        assert_eq!(test_dir_origin_meta.uid(), test_dir_mount_meta.uid());
        // 检查文件所属组是否相同
        assert_eq!(test_dir_origin_meta.gid(), test_dir_mount_meta.gid());
        // 检查文件修改时间是否相同
        assert_eq!(test_dir_origin_meta.mtime(), test_dir_mount_meta.mtime());
        // 检查文件访问时间是否相同
        assert_eq!(test_dir_origin_meta.atime(), test_dir_mount_meta.atime());
        // 检查文件创建时间是否相同
        assert_eq!(test_dir_origin_meta.ctime(), test_dir_mount_meta.ctime());
        // 检查文件类型是否相同
        assert_eq!(
            test_dir_origin_meta.file_type(),
            test_dir_mount_meta.file_type()
        );
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

#[tokio::test]
async fn test_read_file_in_sub_dir() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    // 写入子目录到原始目录
    let test_dir = "test_read_dir/";
    let mut test_dir_origin = origin_path.clone();
    test_dir_origin.push(test_dir);
    fs::create_dir(&test_dir_origin).unwrap();
    // 检查子目录是否存在于原始目录
    assert!(Path::new(&test_dir_origin).exists());

    // 写入文件到子目录
    let test_file = "test_read.txt";
    let mut test_file_origin = test_dir_origin.clone();
    test_file_origin.push(test_file);
    // 写入文件内容
    let content = "Hello, World!";
    let mut file = File::create(&test_file_origin).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    // 检查文件是否存在于原始目录
    assert!(Path::new(&test_file_origin).exists());

    let closure = || {
        // 读取挂载文件内容
        let mut test_file_mount = mount_path.clone();
        test_file_mount.push(test_dir);
        test_file_mount.push(test_file);
        let mut file = File::open(&test_file_mount).unwrap();
        let mut read_content = String::new();
        file.read_to_string(&mut read_content).unwrap();
        // 检查文件内容是否相同
        assert_eq!(content, read_content);

        // 测试meta信息读取是否成功
        let test_file_origin_meta = fs::metadata(&test_file_origin).unwrap();
        let test_file_mount_meta = fs::metadata(&test_file_mount).unwrap();

        assert_ne!(test_file_origin, test_file_mount);

        // 检查inode号是否相同
        assert_eq!(test_file_origin_meta.ino(), test_file_mount_meta.ino());
        // 检查文件大小是否相同
        assert_eq!(test_file_origin_meta.size(), test_file_mount_meta.size());
        // 检查文件权限是否相同
        assert_eq!(test_file_origin_meta.mode(), test_file_mount_meta.mode());
        // 检查文件所有者是否相同
        assert_eq!(test_file_origin_meta.uid(), test_file_mount_meta.uid());
        // 检查文件所属组是否相同
        assert_eq!(test_file_origin_meta.gid(), test_file_mount_meta.gid());
        // 检查文件修改时间是否相同
        assert_eq!(test_file_origin_meta.mtime(), test_file_mount_meta.mtime());
        // 检查文件访问时间是否相同
        // assert_eq!(test_file_origin_meta.atime(), test_file_mount_meta.atime());
        // 检查文件创建时间是否相同
        assert_eq!(test_file_origin_meta.ctime(), test_file_mount_meta.ctime());
        // 检查文件类型是否相同
        assert_eq!(
            test_file_origin_meta.file_type(),
            test_file_mount_meta.file_type()
        );
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

#[tokio::test]
async fn test_read_dir() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    let mut test_file_origin = origin_path.clone();
    // 写入子目录到原始目录
    let test_file = "test_read_dir";
    test_file_origin.push(test_file);
    assert!(fs::create_dir(&test_file_origin).is_ok());
    assert!(Path::new(&test_file_origin).exists());
    assert!(fs::metadata(&test_file_origin).unwrap().is_dir());

    let closure = || {
        // 读取挂载文件内容
        let test_dir_mount = mount_path.clone();
        let entries = fs::read_dir(&test_dir_mount).unwrap();
        // 检查文件夹下是否有文件
        assert_eq!(entries.count(), 1);
        // 检查文件夹下文件是否为子文件夹
        fs::read_dir(&test_dir_mount).unwrap().for_each(|entry| {
            let entry = entry.unwrap();
            let entry_meta = entry.metadata().unwrap();
            assert!(entry_meta.is_dir());
            assert_eq!(entry.file_name(), test_file);
        });
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
