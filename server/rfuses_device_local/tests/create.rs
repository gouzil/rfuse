use std::{
    fs::{self, File},
    os::unix::fs::MetadataExt,
    path::Path,
};

use common::{rfuses_spawn_run, run_command_with_status, TestContext};

mod common;

#[tokio::test]
async fn test_create_file() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    let closure = || {
        // 测试文件写入是否成功
        let test_file = "test_write.txt";
        // 写入文件到挂载目录
        let mut test_file_mount = mount_path.clone();
        test_file_mount.push(test_file);
        // 确认写入文件成功
        assert!(File::create(&test_file_mount).is_ok());
        // 检查文件是否存在于原始目录
        let mut test_file_origin = origin_path.clone();
        test_file_origin.push(test_file);
        assert!(Path::new(&test_file_origin).exists());

        // 测试meta信息写入是否成功
        let test_file_mount_meta = fs::metadata(test_file_mount);
        let test_file_origin_meta = fs::metadata(test_file_origin);
        // 检查meta信息是否正常读取
        assert!(test_file_mount_meta.is_ok());
        assert!(test_file_origin_meta.is_ok());
        // 检查meta信息是否相同
        let test_file_mount_meta = test_file_mount_meta.unwrap();
        let test_file_origin_meta = test_file_origin_meta.unwrap();
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

#[tokio::test]
async fn test_create_dir() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    let closure = || {
        // 测试目录写入是否成功
        let test_dir = "test_dir";
        // 写入目录到挂载目录
        let mut test_dir_mount = mount_path.clone();
        test_dir_mount.push(test_dir);
        // 确认写入目录成功
        assert!(fs::create_dir(&test_dir_mount).is_ok());
        // 检查目录是否存在于原始目录
        let mut test_dir_origin = origin_path.clone();
        test_dir_origin.push(test_dir);
        assert!(Path::new(&test_dir_origin).exists());

        // 测试meta信息写入是否成功
        let test_dir_mount_meta = fs::metadata(test_dir_mount);
        let test_dir_origin_meta = fs::metadata(test_dir_origin);
        // 检查meta信息是否正常读取
        assert!(test_dir_mount_meta.is_ok());
        assert!(test_dir_origin_meta.is_ok());
        // 检查meta信息是否相同
        let test_dir_mount_meta = test_dir_mount_meta.unwrap();
        let test_dir_origin_meta = test_dir_origin_meta.unwrap();
        // 检查目录权限是否相同
        assert_eq!(test_dir_mount_meta.mode(), test_dir_origin_meta.mode());
        // 检查目录所有者是否相同
        assert_eq!(test_dir_mount_meta.uid(), test_dir_origin_meta.uid());
        // 检查目录所属组是否相同
        assert_eq!(test_dir_mount_meta.gid(), test_dir_origin_meta.gid());
        // 检查目录修改时间是否相同
        assert_eq!(test_dir_mount_meta.mtime(), test_dir_origin_meta.mtime());
        // 检查目录访问时间是否相同
        assert_eq!(test_dir_mount_meta.atime(), test_dir_origin_meta.atime());
        // 检查目录创建时间是否相同
        assert_eq!(test_dir_mount_meta.ctime(), test_dir_origin_meta.ctime());
        // 检查目录类型是否相同
        assert_eq!(
            test_dir_mount_meta.file_type(),
            test_dir_origin_meta.file_type()
        );
        // 检查inode号是否相同
        assert_eq!(test_dir_mount_meta.ino(), test_dir_origin_meta.ino());
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
async fn test_create_file_in_dir() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    let closure = || {
        // 测试目录写入是否成功
        let test_dir = "test_dir";
        // 写入目录到挂载目录
        let mut test_dir_mount = mount_path.clone();
        test_dir_mount.push(test_dir);
        // 确认写入目录成功
        assert!(fs::create_dir(&test_dir_mount).is_ok());
        // 检查目录是否存在于原始目录
        let mut test_dir_origin = origin_path.clone();
        test_dir_origin.push(test_dir);
        assert!(Path::new(&test_dir_origin).exists());

        // 测试文件写入是否成功
        let test_file = "test_write.txt";
        // 写入文件到挂载目录
        let mut test_file_mount = test_dir_mount.clone();
        test_file_mount.push(test_file);
        // 确认文件创建成功
        assert!(File::create(&test_file_mount).is_ok());
        // 检查文件是否存在于原始目录
        let mut test_file_origin = test_dir_origin.clone();
        test_file_origin.push(test_file);
        assert!(Path::new(&test_file_origin).exists());

        // 测试meta信息写入是否成功
        let test_file_mount_meta = fs::metadata(test_file_mount);
        let test_file_origin_meta = fs::metadata(test_file_origin);
        // 检查meta信息是否正常读取
        assert!(test_file_mount_meta.is_ok());
        assert!(test_file_origin_meta.is_ok());
        // 检查meta信息是否相同
        let test_file_mount_meta = test_file_mount_meta.unwrap();
        let test_file_origin_meta = test_file_origin_meta.unwrap();
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
                .arg("-v")
        },
        closure
    );
}

#[tokio::test]
async fn test_create_dir_in_dir() {
    let context = TestContext::new();

    let mount_path = context.mount_dir.to_owned();
    let origin_path = context.origin_dir.to_owned();

    let closure = || {
        // 测试目录写入是否成功
        let test_dir = "test_dir";
        // 写入目录到挂载目录
        let mut test_dir_mount = mount_path.clone();
        test_dir_mount.push(test_dir);
        // 确认写入目录成功
        assert!(fs::create_dir(&test_dir_mount).is_ok());
        // 检查目录是否存在于原始目录
        let mut test_dir_origin = origin_path.clone();
        test_dir_origin.push(test_dir);
        assert!(Path::new(&test_dir_origin).exists());

        // 测试目录写入是否成功
        let test_sub_dir = "test_sub_dir";
        // 写入目录到挂载目录
        let mut test_sub_dir_mount = test_dir_mount.clone();
        test_sub_dir_mount.push(test_sub_dir);
        // 确认写入目录成功
        assert!(fs::create_dir(&test_sub_dir_mount).is_ok());
        // 检查目录是否存在于原始目录
        let mut test_sub_dir_origin = test_dir_origin.clone();
        test_sub_dir_origin.push(test_sub_dir);
        assert!(Path::new(&test_sub_dir_origin).exists());

        // 测试meta信息写入是否成功
        let test_sub_dir_mount_meta = fs::metadata(test_sub_dir_mount);
        let test_sub_dir_origin_meta = fs::metadata(test_sub_dir_origin);
        // 检查meta信息是否正常读取
        assert!(test_sub_dir_mount_meta.is_ok());
        assert!(test_sub_dir_origin_meta.is_ok());
        // 检查meta信息是否相同
        let test_sub_dir_mount_meta = test_sub_dir_mount_meta.unwrap();
        let test_sub_dir_origin_meta = test_sub_dir_origin_meta.unwrap();
        // 检查目录权限是否相同
        assert_eq!(
            test_sub_dir_mount_meta.mode(),
            test_sub_dir_origin_meta.mode()
        );
        // 检查目录所有者是否相同
        assert_eq!(
            test_sub_dir_mount_meta.uid(),
            test_sub_dir_origin_meta.uid()
        );
        // 检查目录所属组是否相同
        assert_eq!(
            test_sub_dir_mount_meta.gid(),
            test_sub_dir_origin_meta.gid()
        );
        // 检查目录修改时间是否相同
        assert_eq!(
            test_sub_dir_mount_meta.mtime(),
            test_sub_dir_origin_meta.mtime()
        );
        // 检查目录访问时间是否相同
        assert_eq!(
            test_sub_dir_mount_meta.atime(),
            test_sub_dir_origin_meta.atime()
        );
        // 检查目录创建时间是否相同
        assert_eq!(
            test_sub_dir_mount_meta.ctime(),
            test_sub_dir_origin_meta.ctime()
        );
        // 检查目录类型是否相同
        assert_eq!(
            test_sub_dir_mount_meta.file_type(),
            test_sub_dir_origin_meta.file_type()
        );
        // 检查inode号是否相同
        assert_eq!(
            test_sub_dir_mount_meta.ino(),
            test_sub_dir_origin_meta.ino()
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
