use log::{debug, error};
use nix::sys::{
    stat::{fchmodat, utimensat, FchmodatFlags, Mode, UtimensatFlags},
    time::TimeSpec,
};
use nix::unistd::{chown, truncate, Gid, Uid};
use rfuse_core::tmp_file::TmpFile; // 这里的 TmpFile 不用来做锁定，只是用来传输基本的数据
use rfuse_core::{
    inode::{Inode, InodeAttributes, InodeKind},
    tmp_file::TmpFileError,
    utils::i64_to_system_time,
};
use std::{
    fs,
    io::{Read, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    time::SystemTime,
};

use crate::utils::system_time_to_timespec;

// 这个文件主要是抽象对文件的操作, 方便在后续远程调用时使用同种接口

fn change_time(path: &str, atime: &TimeSpec, mtime: &TimeSpec) -> Result<(), TmpFileError> {
    // 将时间戳应用到文件
    match utimensat(None, path, atime, mtime, UtimensatFlags::FollowSymlink) {
        Ok(_) => {
            debug!("Successfully set file attributes.");
            Ok(())
        }
        Err(e) => {
            error!("[LocalDisk][set_attr] Failed to set file attributes: {}", e);
            Err(TmpFileError::ChangeTimeError)
        }
    }
}

pub fn write(tf: &TmpFile, data: &[u8], write_time: &SystemTime) -> Result<(), TmpFileError> {
    let mut file = fs::OpenOptions::new()
        .append(false)
        .create(true)
        .write(true)
        .truncate(true)
        .open(tf.path.clone() + tf.file_name.as_str())
        .unwrap();
    debug!("write to file: {}", tf.path.clone() + tf.file_name.as_str());
    match file.write_all(data) {
        Ok(()) => {
            debug!("Successfully write {} bytes to the file.", data.len());
        }
        Err(e) => {
            error!("[LocalDisk][write] Failed to write to file: {}", e);
            return Err(TmpFileError::WriteError);
        }
    };

    match change_time(
        &(tf.path.clone() + tf.file_name.as_str()),
        &system_time_to_timespec(write_time),
        &system_time_to_timespec(write_time),
    ) {
        Ok(_) => {}
        Err(e) => return Err(e),
    };
    Ok(())
}

pub fn read_all(tf: &TmpFile) -> Result<Vec<u8>, TmpFileError> {
    let mut file = match fs::OpenOptions::new()
        .read(true)
        .open(tf.path.clone() + tf.file_name.as_str())
    {
        Ok(file) => file,
        Err(e) => {
            error!("[LocalDisk][read_all] Failed to open file: {}", e);
            return Err(TmpFileError::ReadError);
        }
    };
    let mut buf: Vec<u8> = Vec::new();
    match file.read_to_end(&mut buf) {
        Ok(_) => {
            debug!("Successfully read {} bytes from the file.", buf.len());
        }
        Err(e) => {
            error!("[LocalDisk][read_all] Failed to read from file: {}", e);
            return Err(TmpFileError::ReadError);
        }
    };
    Ok(buf)
}

pub fn read_exact(tf: &TmpFile, buf: &mut [u8], offset: u64) -> Result<(), TmpFileError> {
    let mut file = match fs::OpenOptions::new()
        .read(true)
        .open(tf.path.clone() + tf.file_name.as_str())
    {
        Ok(f) => f,
        Err(e) => {
            error!("[LocalDisk][read_exact] Failed to open file: {}", e);
            return Err(TmpFileError::ReadError);
        }
    };
    let fr = file.read_exact(&mut buf[..offset as usize]);
    match fr {
        Ok(()) => {
            debug!("Successfully read {} bytes from the file.", buf.len());
            Ok(())
        }
        Err(e) => {
            error!("[LocalDisk][read_exact] Failed to read from file: {}", e);
            Err(TmpFileError::ReadError)
        }
    }
}

pub fn set_attr(tf: &TmpFile, attr: &InodeAttributes) -> Result<(), TmpFileError> {
    let full_path = tf.path.clone() + &tf.file_name;

    let access_time = system_time_to_timespec(&attr.atime);
    let modification_time = system_time_to_timespec(&attr.mtime);

    // 将时间戳应用到文件
    match change_time(full_path.as_str(), &access_time, &modification_time) {
        Ok(_) => {}
        Err(e) => return Err(e),
    };

    #[cfg(target_os = "linux")]
    let permissions = attr.permissions as u32;

    #[cfg(target_os = "macos")]
    let permissions = attr.permissions;

    // 设置权限
    match fchmodat(
        None,
        full_path.as_str(),
        Mode::from_bits_retain(permissions),
        FchmodatFlags::FollowSymlink,
    ) {
        Ok(_) => {
            debug!("Successfully set file permissions.");
        }
        Err(e) => {
            error!("[LocalDisk][set_attr]Failed to set file permissions: {}", e);
            return Err(TmpFileError::SetAttrError);
        }
    }

    // 设置用户和组
    match chown(
        full_path.as_str(),
        Some(Uid::from_raw(attr.uid)),
        Some(Gid::from_raw(attr.gid)),
    ) {
        Ok(_) => {
            debug!("Successfully set file owner.");
        }
        Err(e) => {
            error!("[LocalDisk][set_attr]Failed to set file owner: {}", e);
            return Err(TmpFileError::SetAttrError);
        }
    };

    // 设置文件大小
    match truncate(full_path.as_str(), attr.size as i64) {
        Ok(_) => {
            debug!("Successfully set file size.");
            Ok(())
        }
        Err(e) => {
            error!("[LocalDisk][set_attr]Failed to set file size: {}", e);
            Err(TmpFileError::SetAttrError)
        }
    }
}

pub fn rename(
    tf: &TmpFile,
    new_path: String,
    rename_time: &SystemTime,
) -> Result<(), TmpFileError> {
    debug!(
        "[LocalDisk][rename] file from {} to {}",
        tf.path.clone() + &tf.file_name,
        new_path
    );
    // 重命名文件
    match fs::rename(tf.path.clone() + &tf.file_name, &new_path) {
        Ok(_) => {}
        Err(e) => {
            error!("[LocalDisk][rename] Failed to rename file: {}", e);
            return Err(TmpFileError::RenameError);
        }
    };

    let file_meta = match fs::metadata(&new_path) {
        Ok(file) => file,
        Err(e) => {
            error!("[LocalDisk][rename] Failed to get file meta: {}", e);
            return Err(TmpFileError::ReadError);
        }
    };

    // 修改重命名后的文件时间
    match change_time(
        new_path.as_str(),
        &system_time_to_timespec(&i64_to_system_time(file_meta.atime())),
        &system_time_to_timespec(rename_time),
    ) {
        Ok(_) => {}
        Err(e) => {
            return Err(e);
        }
    };

    // 将时间戳应用到原地址的上层文件夹
    match change_time(
        tf.path.as_str(),
        &system_time_to_timespec(&i64_to_system_time(file_meta.atime())),
        &system_time_to_timespec(rename_time),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn create_file(tf: &TmpFile) -> Result<Inode, TmpFileError> {
    match fs::File::create(tf.path.clone() + &tf.file_name) {
        Ok(f) => {
            // 这里所有的数据都是临时数据, 会在 RFuseFS 中被使用
            let mut attr =
                InodeAttributes::new(tf.file_name.clone(), InodeKind::File, tf.path.clone());

            let meta = f.metadata().unwrap();
            attr.ctime = i64_to_system_time(meta.ctime());
            attr.mtime = i64_to_system_time(meta.mtime());
            attr.size = meta.size();
            attr.permissions = meta.permissions().mode() as u16;
            attr.uid = meta.uid();
            attr.gid = meta.gid();

            let mut inode = Inode::new(0, attr);
            inode.ino = meta.ino();
            Ok(inode)
        }
        Err(e) => {
            error!("[LocalDisk][create_file] Failed to create file: {}", e);
            Err(TmpFileError::CreateError)
        }
    }
}

pub fn remove_file(tf: &TmpFile, rm_file_time: &SystemTime) -> Result<(), TmpFileError> {
    match fs::remove_file(tf.path.clone() + &tf.file_name) {
        Ok(_) => {
            debug!(
                "[LocalDisk][remove_file] Successfully remove file. {}",
                tf.path.clone() + &tf.file_name
            );
        }
        Err(e) => {
            error!("[LocalDisk][remove_file] Failed to remove file: {}", e);
            return Err(TmpFileError::RemoveError);
        }
    };

    // 将时间戳应用到上层文件夹
    let file_meta = match fs::metadata(tf.path.clone()) {
        Ok(file) => file,
        Err(e) => {
            error!("[LocalDisk][remove_file] Failed to get file meta: {}", e);
            return Err(TmpFileError::ReadError);
        }
    };

    match change_time(
        tf.path.as_str(),
        &system_time_to_timespec(&i64_to_system_time(file_meta.atime())),
        &system_time_to_timespec(rm_file_time),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn make_dir(tf: &TmpFile, mode: u32) -> Result<Inode, TmpFileError> {
    let full_path = tf.path.clone() + &tf.file_name;

    match fs::create_dir(&full_path) {
        Ok(_) => {
            debug!("[LocalDisk][make_dir] Successfully make dir.");
        }
        Err(e) => {
            error!("[LocalDisk][make_dir] Failed to make dir: {}", e);
            return Err(TmpFileError::MakeDirError);
        }
    };
    let meta = match fs::metadata(&full_path) {
        Ok(meta) => meta,
        Err(e) => {
            error!("[LocalDisk][make_dir] Failed to get metadata: {}", e);
            return Err(TmpFileError::MakeDirError);
        }
    };
    let mut perms = meta.permissions();
    perms.set_mode(mode);
    match fs::set_permissions(&full_path, perms) {
        Ok(_) => {
            debug!("[LocalDisk][make_dir] Successfully set dir permissions.");
        }
        Err(e) => {
            error!("[LocalDisk][make_dir] Failed to set dir permissions: {}", e);
            return Err(TmpFileError::MakeDirError);
        }
    };
    match fs::metadata(&full_path) {
        Ok(meta) => {
            // 这里所有的数据都是临时数据, 会在 RFuseFS 中被使用
            let mut attr =
                InodeAttributes::new(tf.file_name.clone(), InodeKind::Directory, tf.path.clone());
            attr.ctime = i64_to_system_time(meta.ctime());
            attr.mtime = i64_to_system_time(meta.mtime());
            attr.size = meta.size();
            attr.permissions = meta.permissions().mode() as u16;
            attr.uid = meta.uid();
            attr.gid = meta.gid();
            let mut inode = Inode::new(0, attr);
            inode.ino = meta.ino();
            Ok(inode)
        }
        Err(e) => {
            error!("[LocalDisk][make_dir] Failed to get metadata: {}", e);
            Err(TmpFileError::MakeDirError)
        }
    }
}

pub fn remove_dir(tf: &TmpFile, rm_dir_time: &SystemTime) -> Result<(), TmpFileError> {
    match fs::remove_dir_all(tf.path.clone() + &tf.file_name) {
        Ok(_) => {
            debug!(
                "[LocalDisk][remove_dir] Successfully remove dir. {}",
                tf.path.clone() + &tf.file_name
            );
        }
        Err(e) => {
            error!("[LocalDisk][remove_dir] Failed to remove dir: {}", e);
            return Err(TmpFileError::RemoveDirError);
        }
    };

    // 将时间戳应用到上层文件夹
    let file_meta = match fs::metadata(tf.path.clone()) {
        Ok(file) => file,
        Err(e) => {
            error!("[LocalDisk][remove_dir] Failed to get meta dir: {}", e);
            return Err(TmpFileError::RemoveDirError);
        }
    };

    match change_time(
        tf.path.as_str(),
        &system_time_to_timespec(&i64_to_system_time(file_meta.atime())),
        &system_time_to_timespec(rm_dir_time),
    ) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
