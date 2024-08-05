use rfuse_device_disk::local_disk;
use std::time::SystemTime;

use rfuse_core::{
    inode::{Inode, InodeAttributes},
    tmp_file::{TmpFile, TmpFileError, TmpFileTrait},
};

pub struct LocalFS;

impl TmpFileTrait for LocalFS {
    fn write(
        &self,
        tf: &TmpFile,
        data: &[u8],
        write_time: &SystemTime,
    ) -> Result<(), TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        local_disk::write(tf, data, write_time)
    }

    fn read_all(&self, tf: &TmpFile) -> Result<Vec<u8>, TmpFileError> {
        let _guard = tf.lock.read().unwrap();
        local_disk::read_all(tf)
    }

    fn read_exact(&self, tf: &TmpFile, buf: &mut [u8], offset: u64) -> Result<(), TmpFileError> {
        let _guard = tf.lock.read().unwrap();
        local_disk::read_exact(tf, buf, offset)
    }

    fn set_attr(&self, tf: &TmpFile, attr: &InodeAttributes) -> Result<(), TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        local_disk::set_attr(tf, attr)
    }

    fn rename(
        &self,
        tf: &TmpFile,
        new_path: String,
        rename_time: &SystemTime,
    ) -> Result<(), TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        local_disk::rename(tf, new_path, rename_time)
    }

    fn create_file(&self, tf: &TmpFile) -> Result<Inode, TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        local_disk::create_file(tf)
    }

    fn remove_file(&self, tf: &TmpFile, rm_file_time: &SystemTime) -> Result<(), TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        local_disk::remove_file(tf, rm_file_time)
    }

    fn make_dir(&self, tf: &TmpFile, mode: u32) -> Result<Inode, TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        local_disk::make_dir(tf, mode)
    }

    fn remove_dir(&self, tf: &TmpFile, rm_dir_time: &SystemTime) -> Result<(), TmpFileError> {
        let _guard = tf.lock.write().unwrap();
        local_disk::remove_dir(tf, rm_dir_time)
    }
}
