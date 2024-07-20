use std::sync::RwLock;

use log::warn;

use crate::inode::{Inode, InodeAttributes};

#[allow(dead_code)]
#[derive(Debug)]
pub struct TmpFile {
    pub file_name: String,
    pub path: String,
    pub lock: RwLock<()>,
}

pub trait TmpFileTrait {
    // 写入文件
    fn write(&mut self, data: &[u8]) -> Result<(), &str> {
        warn!("[TmpFileTrait][Not Implemented] write(data: {:#?})", data);
        Err("write failed")
    }

    // 读取文件所有内容
    fn read_all(&self) -> Result<Vec<u8>, &str> {
        warn!("[TmpFileTrait][Not Implemented] read_all()");
        Err("read_all failed")
    }

    // 读取文件
    fn read_exact(&self, buf: &mut [u8], offset: u64) -> Result<(), &str> {
        warn!(
            "[TmpFileTrait][Not Implemented] read_exact(buf: {:#?}, offset: {:#?})",
            buf, offset
        );
        Err("read_exact failed")
    }

    // 设置属性
    fn set_attr(&self, attr: &InodeAttributes) -> Result<(), &str> {
        warn!(
            "[TmpFileTrait][Not Implemented] set_attr(attr: {:#?})",
            attr
        );
        Err("set_attr failed")
    }

    // 修改文件名
    fn rename(&self, new_path: String) -> Result<(), &str> {
        warn!(
            "[TmpFileTrait][Not Implemented] rename(new_path: {:#?})",
            new_path
        );
        Err("rename failed")
    }

    // 创建文件
    fn create_file(&self) -> Result<Inode, &str> {
        warn!("[TmpFileTrait][Not Implemented] create_file()");
        Err("create_file failed")
    }

    // 删除文件
    fn remove_file(&self) -> Result<(), &str> {
        warn!("[TmpFileTrait][Not Implemented] remove_file()");
        Err("remove_file failed")
    }

    // 创建目录
    fn make_dir(&self, mode: u32) -> Result<Inode, &str> {
        warn!(
            "[TmpFileTrait][Not Implemented] make_dir(mode: {:#?})",
            mode
        );
        Err("make_dir failed")
    }

    // 删除目录
    fn remove_dir(&self) -> Result<(), &str> {
        warn!("[TmpFileTrait][Not Implemented] remove_dir()");
        Err("remove_dir failed")
    }
}

impl TmpFileTrait for TmpFile {}
