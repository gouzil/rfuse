use std::{
    fs::{self, File},
    io::{Read, Write},
    os::unix::fs::MetadataExt,
    path::Path,
};

use common::{rfuses_spawn_run, run_command_with_status, TestContext};

mod common;

#[tokio::test]
async fn test_rename_file() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    // 测试文件重命名是否成功
    let test_file = "test_rename.txt";
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
        // 重命名文件
        let test_file_rename = "test_rename_rename.txt";
        // 新文件地址
        let mut test_file_rename_mount = mount_path.clone();
        test_file_rename_mount.push(test_file_rename);
        // 旧文件地址
        let mut test_file_mount = mount_path.clone();
        test_file_mount.push(test_file);
        fs::rename(&test_file_mount, &test_file_rename_mount).unwrap();
        let mut file = File::open(&test_file_rename_mount).unwrap();
        let mut read_content = String::new();
        file.read_to_string(&mut read_content).unwrap();

        let mut test_file_rename_origin = origin_path.clone();
        test_file_rename_origin.push(test_file_rename);
        // 检查文件是否存在于挂载目录
        assert!(Path::new(&test_file_rename_origin).exists());
        // 检查文件内容是否相同
        assert_eq!(content, read_content);

        // 测试meta信息读取是否成功
        let test_file_origin_meta = fs::metadata(test_file_rename_origin).unwrap();
        let test_file_mount_meta = fs::metadata(test_file_rename_mount).unwrap();

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
        // assert_eq!(test_file_origin_meta.mtime(), test_file_mount_meta.mtime());
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
async fn test_rename_in_mount_file() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    let closure = || {
        // 测试文件重命名是否成功
        let test_file = "test_rename.txt";
        // 写入文件到挂载目录
        let mut test_file_mount = mount_path.clone();
        test_file_mount.push(test_file);
        // 写入文件内容
        let content = "Hello, World!";
        let mut file = File::create(&test_file_mount).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        // 检查文件是否存在于挂载目录
        assert!(Path::new(&test_file_mount).exists());

        // 重命名文件
        let test_file_rename = "test_rename_rename.txt";
        let mut test_file_rename_mount = mount_path.clone();
        test_file_rename_mount.push(test_file_rename);
        fs::rename(&test_file_mount, &test_file_rename_mount).unwrap();
        // 检查文件是否存在于挂载目录
        assert!(Path::new(&test_file_rename_mount).exists());

        // 读取原始文件内容
        let mut test_file_origin = origin_path.clone();
        test_file_origin.push(test_file_rename);
        let mut file = File::open(&test_file_origin).unwrap();
        let mut read_content = String::new();
        file.read_to_string(&mut read_content).unwrap();
        // 检查文件内容是否相同
        assert_eq!(content, read_content);

        // 测试meta信息读取是否成功
        let test_file_mount_meta = fs::metadata(test_file_rename_mount).unwrap();
        let test_file_origin_meta = fs::metadata(test_file_origin).unwrap();

        // 检查inode号是否相同
        assert_eq!(test_file_mount_meta.ino(), test_file_origin_meta.ino());
        // 检查文件大小是否相同
        assert_eq!(test_file_mount_meta.size(), test_file_origin_meta.size());
        // 检查文件权限是否相同
        assert_eq!(test_file_mount_meta.mode(), test_file_origin_meta.mode());
        // 检查文件所有者是否相同
        assert_eq!(test_file_mount_meta.uid(), test_file_origin_meta.uid());
        // 检查文件所属组是否相同
        assert_eq!(test_file_mount_meta.gid(), test_file_origin_meta.gid());
        // 检查文件修改时间是否相同
        // assert_eq!(test_file_mount_meta.mtime(), test_file_origin_meta.mtime());
        // 检查文件访问时间是否相同
        // assert_eq!(test_file_mount_meta.atime(), test_file_origin_meta.atime());
        // 检查文件创建时间是否相同
        assert_eq!(test_file_mount_meta.ctime(), test_file_origin_meta.ctime());
        // 检查文件类型是否相同
        assert_eq!(
            test_file_mount_meta.file_type(),
            test_file_origin_meta.file_type()
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
async fn test_rename_dir() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    // 测试文件夹重命名是否成功
    let test_dir = "test_rename_dir/";
    // 创建文件夹到原始目录
    let mut test_dir_origin = origin_path.clone();
    test_dir_origin.push(test_dir);
    fs::create_dir(&test_dir_origin).unwrap();
    // 检查文件夹是否存在于原始目录
    assert!(Path::new(&test_dir_origin).exists());

    let closure = || {
        // 重命名文件夹
        let test_dir_rename = "test_rename_dir_rename/";
        // 新文件夹地址
        let mut test_dir_rename_mount = mount_path.clone();
        test_dir_rename_mount.push(test_dir_rename);
        // 旧文件夹地址
        let mut test_dir_mount = mount_path.clone();
        test_dir_mount.push(test_dir);
        fs::rename(&test_dir_mount, &test_dir_rename_mount).unwrap();
        // 检查文件夹是否存在于挂载目录
        assert!(Path::new(&test_dir_rename_mount).exists());

        // 源文件夹位置
        let mut test_dir_rename_origin = origin_path.clone();
        test_dir_rename_origin.push(test_dir_rename);
        // 检查文件夹是否存在于原始目录
        assert!(Path::new(&test_dir_rename_origin).exists());

        // 测试meta信息读取是否成功
        let test_dir_origin_meta = fs::metadata(&test_dir_rename_origin).unwrap();
        let test_dir_mount_meta = fs::metadata(&test_dir_rename_mount).unwrap();

        assert_ne!(test_dir_rename_origin, test_dir_rename_mount);

        // 检查inode号是否相同
        assert_eq!(test_dir_origin_meta.ino(), test_dir_mount_meta.ino());
        // 检查文件夹大小是否相同
        assert_eq!(test_dir_origin_meta.size(), test_dir_mount_meta.size());
        // 检查文件夹权限是否相同
        assert_eq!(test_dir_origin_meta.mode(), test_dir_mount_meta.mode());
        // 检查文件夹所有者是否相同
        assert_eq!(test_dir_origin_meta.uid(), test_dir_mount_meta.uid());
        // 检查文件夹所属组是否相同
        assert_eq!(test_dir_origin_meta.gid(), test_dir_mount_meta.gid());
        // 检查文件夹修改时间是否相同
        // assert_eq!(test_dir_origin_meta.mtime(), test_dir_mount_meta.mtime());
        // 检查文件夹访问时间是否相同
        // assert_eq!(test_dir_origin_meta.atime(), test_dir_mount_meta.atime());
        // 检查文件夹创建时间是否相同
        assert_eq!(test_dir_origin_meta.ctime(), test_dir_mount_meta.ctime());
        // 检查文件夹类型是否相同
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
async fn test_rename_in_mount_dir() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    let closure = || {
        // 测试文件夹重命名是否成功
        let test_dir = "test_rename_dir";
        // 创建文件夹到挂载目录
        let mut test_dir_mount = mount_path.clone();
        test_dir_mount.push(test_dir);
        fs::create_dir(&test_dir_mount).unwrap();
        // 检查文件夹是否存在于挂载目录
        assert!(Path::new(&test_dir_mount).exists());

        // 重命名文件夹
        let test_dir_rename = "test_rename_dir_rename";
        let mut test_dir_rename_mount = mount_path.clone();
        test_dir_rename_mount.push(test_dir_rename);
        fs::rename(&test_dir_mount, &test_dir_rename_mount).unwrap();
        // 检查文件夹是否存在于挂载目录
        assert!(Path::new(&test_dir_rename_mount).exists());

        // 源文件夹位置
        let mut test_dir_origin = origin_path.clone();
        test_dir_origin.push(test_dir_rename);
        // 检查文件夹是否存在于原始目录
        assert!(Path::new(&test_dir_origin).exists());

        // 测试meta信息读取是否成功
        let test_dir_mount_meta = fs::metadata(test_dir_rename_mount).unwrap();
        let test_dir_origin_meta = fs::metadata(test_dir_origin).unwrap();

        // 检查inode号是否相同
        assert_eq!(test_dir_mount_meta.ino(), test_dir_origin_meta.ino());
        // 检查文件夹大小是否相同
        assert_eq!(test_dir_mount_meta.size(), test_dir_origin_meta.size());
        // 检查文件夹权限是否相同
        assert_eq!(test_dir_mount_meta.mode(), test_dir_origin_meta.mode());
        // 检查文件夹所有者是否相同
        assert_eq!(test_dir_mount_meta.uid(), test_dir_origin_meta.uid());
        // 检查文件夹所属组是否相同
        assert_eq!(test_dir_mount_meta.gid(), test_dir_origin_meta.gid());
        // 检查文件夹修改时间是否相同
        assert_eq!(test_dir_mount_meta.mtime(), test_dir_origin_meta.mtime());
        // 检查文件夹访问时间是否相同
        // assert_eq!(test_dir_mount_meta.atime(), test_dir_origin_meta.atime());
        // 检查文件夹创建时间是否相同
        assert_eq!(test_dir_mount_meta.ctime(), test_dir_origin_meta.ctime());
        // 检查文件夹类型是否相同
        assert_eq!(
            test_dir_mount_meta.file_type(),
            test_dir_origin_meta.file_type()
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
