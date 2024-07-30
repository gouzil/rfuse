use core::fmt;
use std::{sync::RwLock, time::SystemTime};

use log::warn;

use crate::inode::{Inode, InodeAttributes};

pub enum TmpFileError {
    ReadError,
    WriteError,
    RenameError,
    RemoveError,
    CreateError,
    SetAttrError,
    MakeDirError,
    RemoveDirError,
    ChangeTimeError,
}

impl fmt::Display for TmpFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            TmpFileError::ReadError => write!(f, "ReadError"),
            TmpFileError::WriteError => write!(f, "WriteError"),
            TmpFileError::RenameError => write!(f, "RenameError"),
            TmpFileError::RemoveError => write!(f, "RemoveError"),
            TmpFileError::CreateError => write!(f, "CreateError"),
            TmpFileError::SetAttrError => write!(f, "SetAttrError"),
            TmpFileError::MakeDirError => write!(f, "MakeDirError"),
            TmpFileError::RemoveDirError => write!(f, "RemoveDirError"),
            TmpFileError::ChangeTimeError => write!(f, "ChangeTimeError"),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct TmpFile {
    pub file_name: String,
    pub path: String,
    pub lock: RwLock<()>,
}

pub trait TmpFileTrait {
    // 写入文件
    fn write(&self, tf: &TmpFile, data: &[u8], write_time: SystemTime) -> Result<(), TmpFileError> {
        warn!(
            "[TmpFileTrait][Not Implemented] node: {:#?}, write(data: {:#?}), write_time: {:#?}",
            tf, data, write_time
        );
        Err(TmpFileError::WriteError)
    }

    // 读取文件所有内容
    fn read_all(&self, tf: &TmpFile) -> Result<Vec<u8>, TmpFileError> {
        warn!(
            "[TmpFileTrait][Not Implemented] node: {:#?}, read_all()",
            tf
        );
        Err(TmpFileError::ReadError)
    }

    // 读取文件
    fn read_exact(&self, tf: &TmpFile, buf: &mut [u8], offset: u64) -> Result<(), TmpFileError> {
        warn!(
            "[TmpFileTrait][Not Implemented] node: {:#?}, read_exact(buf: {:#?}, offset: {:#?})",
            tf, buf, offset
        );
        Err(TmpFileError::ReadError)
    }

    // 设置属性
    fn set_attr(&self, tf: &TmpFile, attr: &InodeAttributes) -> Result<(), TmpFileError> {
        warn!(
            "[TmpFileTrait][Not Implemented] node: {:#?}, set_attr(attr: {:#?})",
            tf, attr
        );
        Err(TmpFileError::SetAttrError)
    }

    // 修改文件名
    fn rename(&self, tf: &TmpFile, new_path: String) -> Result<(), TmpFileError> {
        warn!(
            "[TmpFileTrait][Not Implemented] node: {:#?}, rename(new_path: {:#?})",
            tf, new_path
        );
        Err(TmpFileError::RenameError)
    }

    // 创建文件
    fn create_file(&self, tf: &TmpFile) -> Result<Inode, TmpFileError> {
        warn!(
            "[TmpFileTrait][Not Implemented] node: {:#?}, create_file()",
            tf
        );
        Err(TmpFileError::CreateError)
    }

    // 删除文件
    fn remove_file(&self, tf: &TmpFile, rm_file_time: SystemTime) -> Result<(), TmpFileError> {
        warn!(
            "[TmpFileTrait][Not Implemented] node: {:#?}, remove_file(rm_file_time: {:#?})",
            tf, rm_file_time
        );
        Err(TmpFileError::RemoveError)
    }

    // 创建目录
    fn make_dir(&self, tf: &TmpFile, mode: u32) -> Result<Inode, TmpFileError> {
        warn!(
            "[TmpFileTrait][Not Implemented] node: {:#?}, make_dir(mode: {:#?})",
            tf, mode
        );
        Err(TmpFileError::MakeDirError)
    }

    // 删除目录
    fn remove_dir(&self, tf: &TmpFile, rm_dir_time: SystemTime) -> Result<(), TmpFileError> {
        warn!(
            "[TmpFileTrait][Not Implemented] node: {:#?}, remove_dir(rm_dir_time: {:#?})",
            tf, rm_dir_time
        );
        Err(TmpFileError::RemoveDirError)
    }
}
