use std::{
    fs,
    io::{Read, Write},
    os::unix::fs::{MetadataExt, PermissionsExt},
    time::SystemTime,
};

use log::{debug, error};
use nix::sys::{
    stat::{fchmodat, utimensat, FchmodatFlags, Mode, UtimensatFlags},
    time::TimeSpec,
};

use crate::utils::system_time_to_timespec;
use nix::unistd::{chown, truncate, Gid, Uid};
use rfuse_core::{
    inode::{Inode, InodeAttributes, InodeKind},
    tmp_file::{TmpFile, TmpFileError, TmpFileTrait},
    utils::i64_to_system_time,
};

fn change_time(path: &str, atime: &TimeSpec, mtime: &TimeSpec) -> Result<(), TmpFileError> {
    // 将时间戳应用到文件
    match utimensat(None, path, atime, mtime, UtimensatFlags::NoFollowSymlink) {
        Ok(_) => {
            debug!("Successfully set file attributes.");
            Ok(())
        }
        Err(e) => {
            error!("[LocalFS][set_attr] Failed to set file attributes: {}", e);
            Err(TmpFileError::ChangeTimeError)
        }
    }
}

pub struct LocalFS;

impl TmpFileTrait for LocalFS {
    fn write(&self, tf: &TmpFile, data: &[u8], write_time: SystemTime) -> Result<(), TmpFileError> {
        let _guard = tf.lock.write().unwrap();
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
                error!("[LocalFS][write] Failed to write to file: {}", e);
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

    fn read_all(&self, tf: &TmpFile) -> Result<Vec<u8>, TmpFileError> {
        let _guard = tf.lock.read().unwrap();
        let mut file = match fs::OpenOptions::new()
            .read(true)
            .open(tf.path.clone() + tf.file_name.as_str())
        {
            Ok(file) => file,
            Err(e) => {
                error!("[LocalFS][read_all] Failed to open file: {}", e);
                return Err(TmpFileError::ReadError);
            }
        };
        let mut buf: Vec<u8> = Vec::new();
        match file.read_to_end(&mut buf) {
            Ok(_) => {
                debug!("Successfully read {} bytes from the file.", buf.len());
            }
            Err(e) => {
                error!("[LocalFS][read_all] Failed to read from file: {}", e);
                return Err(TmpFileError::ReadError);
            }
        };
        Ok(buf)
    }

    fn read_exact(&self, tf: &TmpFile, buf: &mut [u8], offset: u64) -> Result<(), TmpFileError> {
        let _guard = tf.lock.read().unwrap();
        let mut file = match fs::OpenOptions::new()
            .read(true)
            .open(tf.path.clone() + tf.file_name.as_str())
        {
            Ok(f) => f,
            Err(e) => {
                error!("[LocalFS][read_exact] Failed to open file: {}", e);
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
                error!("[LocalFS][read_exact] Failed to read from file: {}", e);
                Err(TmpFileError::ReadError)
            }
        }
    }

    fn set_attr(&self, tf: &TmpFile, attr: &InodeAttributes) -> Result<(), TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        let full_path = tf.path.clone() + &tf.file_name;

        let access_time = system_time_to_timespec(attr.atime);
        let modification_time = system_time_to_timespec(attr.mtime);

        // 将时间戳应用到文件
        match change_time(full_path.as_str(), &access_time, &modification_time) {
            Ok(_) => {}
            Err(e) => return Err(e),
        };

        // 设置权限
        match fchmodat(
            None,
            full_path.as_str(),
            Mode::from_bits_retain(attr.permissions as u32),
            FchmodatFlags::FollowSymlink,
        ) {
            Ok(_) => {
                debug!("Successfully set file permissions.");
            }
            Err(e) => {
                error!("[LocalFS][set_attr]Failed to set file permissions: {}", e);
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
                error!("[LocalFS][set_attr]Failed to set file owner: {}", e);
                return Err(TmpFileError::SetAttrError);
            }
        };

        // 设置文件大小
        match truncate(full_path.as_str(), attr.size as i64) {
            Ok(_) => {
                debug!("Successfully set file size.");
            }
            Err(e) => {
                error!("[LocalFS][set_attr]Failed to set file size: {}", e);
                return Err(TmpFileError::SetAttrError);
            }
        };

        Ok(())
    }

    fn rename(&self, tf: &TmpFile, new_path: String) -> Result<(), TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        debug!(
            "[LocalFS][rename] file from {} to {}",
            tf.path.clone() + &tf.file_name,
            new_path
        );
        match fs::rename(tf.path.clone() + &tf.file_name, new_path) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("[LocalFS][rename] Failed to rename file: {}", e);
                Err(TmpFileError::RenameError)
            }
        }
    }

    fn create_file(&self, tf: &TmpFile) -> Result<Inode, TmpFileError> {
        let _guard = tf.lock.write().unwrap();
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
                error!("[LocalFS][create_file] Failed to create file: {}", e);
                Err(TmpFileError::CreateError)
            }
        }
    }

    fn remove_file(&self, tf: &TmpFile) -> Result<(), TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        match fs::remove_file(tf.path.clone() + &tf.file_name) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("[LocalFS][remove_file] Failed to remove file: {}", e);
                Err(TmpFileError::RemoveError)
            }
        }
    }

    fn make_dir(&self, tf: &TmpFile, mode: u32) -> Result<Inode, TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        let full_path = tf.path.clone() + &tf.file_name;

        match fs::create_dir(&full_path) {
            Ok(_) => {
                debug!("[LocalFS][make_dir] Successfully make dir.");
            }
            Err(e) => {
                error!("[LocalFS][make_dir] Failed to make dir: {}", e);
                return Err(TmpFileError::MakeDirError);
            }
        };
        let meta = match fs::metadata(&full_path) {
            Ok(meta) => meta,
            Err(e) => {
                error!("[LocalFS][make_dir] Failed to get metadata: {}", e);
                return Err(TmpFileError::MakeDirError);
            }
        };
        let mut perms = meta.permissions();
        perms.set_mode(mode);
        match fs::set_permissions(&full_path, perms) {
            Ok(_) => {
                debug!("[LocalFS][make_dir] Successfully set dir permissions.");
            }
            Err(e) => {
                error!("[LocalFS][make_dir] Failed to set dir permissions: {}", e);
                return Err(TmpFileError::MakeDirError);
            }
        };
        match fs::metadata(&full_path) {
            Ok(meta) => {
                // 这里所有的数据都是临时数据, 会在 RFuseFS 中被使用
                let mut attr = InodeAttributes::new(
                    tf.file_name.clone(),
                    InodeKind::Directory,
                    tf.path.clone(),
                );
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
                error!("[LocalFS][make_dir] Failed to get metadata: {}", e);
                Err(TmpFileError::MakeDirError)
            }
        }
    }

    fn remove_dir(&self, tf: &TmpFile) -> Result<(), TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        match fs::remove_dir_all(tf.path.clone() + &tf.file_name) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("[LocalFS][remove_dir] Failed to remove dir: {}", e);
                Err(TmpFileError::RemoveDirError)
            }
        }
    }
}
