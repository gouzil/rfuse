use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::MetadataExt,
    path::Path,
};

use common::{rfuses_spawn_run, run_command_with_status, TestContext};

mod common;

#[tokio::test]
async fn test_del_file() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    // 创建测试文件
    let test_file = "test_del.txt";
    // 写入文件到原始目录
    let mut test_file_origin = origin_path.clone();
    test_file_origin.push(test_file);
    // 写入文件内容
    let content = "Hello, World!";
    let mut file = File::create(&test_file_origin).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    // 检查文件是否存在于原始目录
    assert!(Path::new(&test_file_origin).exists());

    let mut closure = || {
        // 检查文件是否存在于挂载目录
        let mut test_file_mount = mount_path.clone();
        test_file_mount.push(test_file);
        assert!(Path::new(&test_file_mount).exists());

        // 删除文件
        fs::remove_file(&test_file_mount).unwrap();

        // 检查文件是否存在于原始目录
        assert!(!Path::new(&test_file_origin).exists());
        // 检查文件是否存在于挂载目录
        assert!(!Path::new(&test_file_mount).exists());

        test_file_mount.pop();
        test_file_origin.pop();

        // 检查上层文件夹
        assert!(Path::new(&test_file_mount).exists());
        assert!(Path::new(&test_file_origin).exists());
        // 检查时间
        let origin_meta = fs::metadata(&test_file_origin).unwrap();
        let mount_meta = fs::metadata(&test_file_mount).unwrap();
        // 检查ctime
        assert_eq!(origin_meta.ctime(), mount_meta.ctime());
        // 检查mtime
        assert_eq!(origin_meta.mtime(), mount_meta.mtime());
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
async fn test_del_dir() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    // 创建测试文件夹
    let test_dir = "test_del_dir/";
    // 创建文件夹到原始目录
    let mut test_dir_origin = origin_path.clone();
    test_dir_origin.push(test_dir);
    fs::create_dir(&test_dir_origin).unwrap();
    // 检查文件夹是否存在于原始目录
    assert!(Path::new(&test_dir_origin).exists());

    let mut closure = || {
        // 检查文件夹是否存在于挂载目录
        let mut test_dir_mount = mount_path.clone();
        test_dir_mount.push(test_dir);
        assert!(Path::new(&test_dir_mount).exists());

        // 删除文件夹
        fs::remove_dir(&test_dir_mount).unwrap();

        // 检查文件夹是否存在于原始目录
        assert!(!Path::new(&test_dir_origin).exists());
        // 检查文件夹是否存在于挂载目录
        assert!(!Path::new(&test_dir_mount).exists());

        test_dir_mount.pop();
        test_dir_origin.pop();

        // 检查上层文件夹
        assert!(Path::new(&test_dir_mount).exists());
        assert!(Path::new(&test_dir_origin).exists());
        // 检查时间
        let origin_meta = fs::metadata(&test_dir_origin).unwrap();
        let mount_meta = fs::metadata(&test_dir_mount).unwrap();
        // 检查ctime
        assert_eq!(origin_meta.ctime(), mount_meta.ctime());
        // 检查mtime
        assert_eq!(origin_meta.mtime(), mount_meta.mtime());
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
async fn test_del_file_in_dir() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    // 创建测试文件夹
    let test_dir = "test_del_dir/";
    // 创建文件夹到原始目录
    let mut test_file_origin = origin_path.clone();
    test_file_origin.push(test_dir);
    fs::create_dir(&test_file_origin).unwrap();
    // 检查文件夹是否存在于原始目录
    assert!(Path::new(&test_file_origin).exists());

    // 创建测试文件
    let test_file = "test_del.txt";
    // 写入文件到原始目录
    test_file_origin.push(test_file);
    // 写入文件内容
    let content = "Hello, World!";
    let mut file = File::create(&test_file_origin).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    // 检查文件是否存在于原始目录
    assert!(Path::new(&test_file_origin).exists());

    let mut closure = || {
        // 检查文件是否存在于挂载目录
        let mut test_file_mount = mount_path.clone();
        test_file_mount.push(test_dir);
        // test_file_mount.push(test_file);
        assert!(Path::new(&test_file_mount).exists());

        // 删除文件
        // ErrorKind::DirectoryNotEmpty
        let result = fs::remove_dir(&test_file_mount);
        assert!(result.is_err());
        // assert!(matches!(
        //     result.err().unwrap().kind(),
        //     ErrorKind::DirectoryNotEmpty
        // ));

        assert!(fs::remove_dir_all(&test_file_mount).is_ok());

        // 检查文件是否存在于原始目录
        assert!(!Path::new(&test_file_origin).exists());
        // 检查文件是否存在于挂载目录
        test_file_origin.pop();
        assert!(!Path::new(&test_file_mount).exists());

        test_file_mount.pop();
        test_file_origin.pop();

        // 检查上层文件夹
        assert!(Path::new(&test_file_mount).exists());
        assert!(Path::new(&test_file_origin).exists());
        // 检查时间
        let origin_meta = fs::metadata(&test_file_origin).unwrap();
        let mount_meta = fs::metadata(&test_file_mount).unwrap();
        // 检查ctime
        assert_eq!(origin_meta.ctime(), mount_meta.ctime());
        // 检查mtime
        assert_eq!(origin_meta.mtime(), mount_meta.mtime());
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
async fn test_clean_file_content() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    // 创建测试文件
    let test_file = "test_clean.txt";
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
        // 检查文件是否存在于挂载目录
        let mut test_file_mount = mount_path.clone();
        test_file_mount.push(test_file);
        assert!(Path::new(&test_file_mount).exists());

        // 清空文件内容
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .truncate(true)
            .open(&test_file_mount)
            .unwrap();
        file.write_all(&[]).unwrap();

        // 检查文件是否存在于原始目录
        assert!(Path::new(&test_file_origin).exists());
        // 检查文件是否存在于挂载目录
        assert!(Path::new(&test_file_mount).exists());

        // 检查时间
        let origin_meta = fs::metadata(&test_file_origin);
        let mount_meta = fs::metadata(&test_file_mount);

        // 检查meta信息是否正常读取
        assert!(origin_meta.is_ok());
        assert!(mount_meta.is_ok());
        // 检查meta信息是否相同
        let test_file_origin_meta = origin_meta.unwrap();
        let test_file_mount_meta = mount_meta.unwrap();
        // 检查ino是否相同
        assert_eq!(test_file_mount_meta.ino(), test_file_origin_meta.ino());
        // 检查目录权限是否相同
        assert_eq!(test_file_mount_meta.mode(), test_file_origin_meta.mode());
        // 检查目录所有者是否相同
        assert_eq!(test_file_mount_meta.uid(), test_file_origin_meta.uid());
        // 检查目录所属组是否相同
        assert_eq!(test_file_mount_meta.gid(), test_file_origin_meta.gid());
        // 检查目录修改时间是否相同
        assert_eq!(test_file_mount_meta.mtime(), test_file_origin_meta.mtime());
        // 检查目录访问时间是否相同
        assert_eq!(test_file_mount_meta.atime(), test_file_origin_meta.atime());
        // // 检查目录创建时间是否相同
        // assert_eq!(test_file_mount_meta.ctime(), test_file_origin_meta.ctime());
        // 检查目录类型是否相同
        assert_eq!(
            test_file_mount_meta.file_type(),
            test_file_origin_meta.file_type()
        );
        // 检查目录大小是否相同
        assert_eq!(test_file_mount_meta.size(), test_file_origin_meta.size());
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
