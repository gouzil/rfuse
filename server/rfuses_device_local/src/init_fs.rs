use std::{
    collections::HashMap,
    os::unix::fs::{MetadataExt, PermissionsExt},
    path::Path,
};

use fuser::FUSE_ROOT_ID;
use log::{debug, info};
use rfuse_core::{
    inode::{root_node, Inode, InodeAttributes, InodeKind},
    remote_fs::{RemoteFileInitializeError, RemoteFileManager},
    utils::i64_to_system_time,
};
use walkdir::{DirEntryExt, WalkDir};

// 用户自定义的初始化函数
pub fn user_defined_init_fs(
    file_manager: &mut RemoteFileManager,
    inodes: &mut HashMap<u64, Inode>,
    source_dir: String,
) -> Result<(), RemoteFileInitializeError> {
    let source_dir_matedata = Path::new(&source_dir).metadata().unwrap();

    // 先插入挂载根节点
    inodes.insert(
        FUSE_ROOT_ID,
        root_node(
            "",
            // self.mountpoint.clone(),
            "/".to_string(),
            source_dir_matedata.permissions().mode() as u16,
            source_dir_matedata.uid(),
            source_dir_matedata.gid(),
        ),
    );

    // 插入根节点下的所有文件
    for entry in WalkDir::new(&source_dir)
        .max_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path() != Path::new(&source_dir))
    {
        let root = inodes.get_mut(&FUSE_ROOT_ID).unwrap();
        root.insert_child(entry.ino());

        // 将原始文件插入到缓存中
        file_manager.add_file(
            entry.ino(),
            entry.file_name().to_str().unwrap().to_string(),
            root.attr.path.clone(),
        );
    }

    // 遍历文件夹下的文件
    for entry in WalkDir::new(&source_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path() != Path::new(&source_dir))
    {
        let path = entry
            .path()
            .display()
            .to_string()
            .replace(&source_dir, "")
            .replace(&entry.file_name().to_str().unwrap().to_string(), "");
        debug!(
            "mount_file: {:?}",
            path.clone() + entry.file_name().to_str().unwrap()
        );

        // 这里插入子文件夹下所有文件的 ino，因为只有文件夹才有子文件，所以在下面那个函数插入
        let mut children_ino = Vec::new();

        // 文件类型
        let kind = if entry.file_type().is_dir() {
            for sub_entry in WalkDir::new(entry.path())
                .max_depth(1)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.path() != Path::new(&entry.path()))
            {
                children_ino.push(sub_entry.ino());
            }
            InodeKind::Directory
        } else {
            InodeKind::File
        };

        let entry_metadata = entry.metadata().unwrap();
        // 添加当前文件夹下的文件到 nodes map 中
        let v = Inode {
            ino: entry.ino(),
            parent_ino: FUSE_ROOT_ID,
            children_ino,
            attr: InodeAttributes {
                id: entry.file_name().to_str().unwrap().to_string(),
                size: entry_metadata.size(),
                name: entry.file_name().to_str().unwrap().to_string(),
                path: path.clone(),
                kind,
                atime: i64_to_system_time(entry_metadata.atime()),
                mtime: i64_to_system_time(entry_metadata.mtime()),
                ctime: i64_to_system_time(entry_metadata.ctime()),
                permissions: entry_metadata.permissions().mode() as u16,
                uid: entry_metadata.uid(),
                gid: entry_metadata.gid(),
            },
        };
        let k = entry.ino();
        debug!("insert ino: {:?}", k);
        debug!("insert value: {:?}", v);
        inodes.insert(k, v.clone());
        file_manager.add_file(k, v.attr.name, source_dir.clone() + &v.attr.path);
    }

    info!("File system init success.");
    Ok(())
}
