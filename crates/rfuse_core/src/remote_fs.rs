use core::fmt;
use std::{collections::HashMap, sync::RwLock, time::SystemTime};

use log::{debug, error};

use crate::{
    inode::{Inode, InodeAttributes},
    tmp_file::{TmpFile, TmpFileTrait},
};

pub type InitFsFuncType = dyn Fn(
        &mut RemoteFileManager,
        &mut HashMap<u64, Inode>,
        String,
    ) -> Result<(), RemoteFileInitializeError>
    + 'static
    + Send;

pub enum RemoteFileInitializeError {
    PermissionError,
    Error,
}

impl fmt::Display for RemoteFileInitializeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RemoteFileInitializeError::PermissionError => write!(f, "PermissionError"),
            RemoteFileInitializeError::Error => write!(f, "Error"),
        }
    }
}

pub struct RemoteFileManager {
    pub tmp_file_map: HashMap<u64, TmpFile>,
    pub init_fs: Box<InitFsFuncType>,
    pub tmp_file_trait: Box<dyn TmpFileTrait>,
}

impl RemoteFileManager {
    pub fn new(init_fs: Box<InitFsFuncType>, tmp_file_trait: Box<dyn TmpFileTrait>) -> Self {
        RemoteFileManager {
            tmp_file_map: HashMap::new(),
            init_fs,
            tmp_file_trait,
        }
    }

    pub fn exist(&self, ino: u64) -> bool {
        self.tmp_file_map.contains_key(&ino)
    }

    pub fn add_file(&mut self, ino: u64, file_name: String, path: String) {
        self.tmp_file_map.insert(
            ino,
            TmpFile {
                file_name,
                path,
                lock: RwLock::new(()),
            },
        );
    }

    pub fn read(&self, ino: u64, buf: &mut [u8], offset: u64) -> Result<(), &str> {
        let inode = match self.tmp_file_map.get(&ino) {
            Some(i) => i,
            None => {
                error!("[RemoteFileManager][read]file not found, ino: {}", ino);
                return Err("file not found");
            }
        };
        match self.tmp_file_trait.read_exact(inode, buf, offset) {
            Ok(_) => {}
            Err(e) => {
                error!("[RemoteFileManager][read] failed: {}", e);
                return Err("read file failed");
            }
        };
        Ok(())
    }

    pub fn read_all(&self, ino: u64) -> Vec<u8> {
        let inode = match self.tmp_file_map.get(&ino) {
            Some(i) => i,
            None => {
                error!("[RemoteFileManager][read_all]file not found, ino: {}", ino);
                return Vec::new();
            }
        };
        match self.tmp_file_trait.read_all(inode) {
            Ok(data) => data,
            Err(e) => {
                error!("[RemoteFileManager][read_all] failed: {}", e);
                Vec::new() // Return an empty Vec<u8> in case of error
            }
        }
    }

    pub fn write_file(
        &mut self,
        ino: u64,
        data: &[u8],
        write_time: &SystemTime,
    ) -> Result<(), &str> {
        let tmp = match self.tmp_file_map.get(&ino) {
            Some(t) => t,
            None => {
                error!(
                    "[RemoteFileManager][write_file]file not found, ino: {}",
                    ino
                );
                return Err("file not found");
            }
        };
        match self.tmp_file_trait.write(tmp, data, write_time) {
            Ok(_) => {}
            Err(e) => {
                error!("[RemoteFileManager][write_file] failed: {}", e);
                return Err("write file failed");
            }
        };
        Ok(())
    }

    pub fn new_file(
        &mut self,
        ino: u64,
        attr: InodeAttributes,
        path: String,
    ) -> Result<Inode, &str> {
        debug!("[RemoteFileManager][new_file] ino: {}, path: {}", ino, path);
        let inode = TmpFile {
            file_name: attr.name.clone(),
            path: path.clone(),
            lock: RwLock::new(()),
        };

        let meta = match self.tmp_file_trait.create_file(&inode) {
            Ok(m) => m,
            Err(e) => {
                error!("[RemoteFileManager][new_file]create file failed: {}", e);
                return Err("create file failed");
            }
        };
        self.tmp_file_map.insert(meta.ino(), inode);
        Ok(meta)
    }

    pub fn set_attr(&mut self, ino: u64, attr: &InodeAttributes) -> Result<(), &str> {
        let inode = match self.tmp_file_map.get_mut(&ino) {
            Some(t) => t,
            None => {
                error!("[RemoteFileManager][set_attr]file not found, ino: {}", ino);
                return Err("file not found");
            }
        };
        match self.tmp_file_trait.set_attr(inode, attr) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("[RemoteFileManager][set_attr] failed: {}", e);
                Err("set attr failed")
            }
        }
    }

    pub fn rename(
        &mut self,
        ino: u64,
        new_name: String,
        new_path: String,
        rename_time: &SystemTime,
    ) -> Result<(), &str> {
        let inode = match self.tmp_file_map.get_mut(&ino) {
            Some(t) => t,
            None => {
                error!("[RemoteFileManager][rename]file not found, ino: {}", ino);
                return Err("file not found");
            }
        };
        match self
            .tmp_file_trait
            .rename(inode, new_path.clone() + new_name.as_str(), rename_time)
        {
            Ok(_) => {}
            Err(e) => {
                error!("[RemoteFileManager][rename] failed: {}", e);
                return Err("rename failed");
            }
        };
        inode.file_name = new_name;
        inode.path = new_path;
        Ok(())
    }

    pub fn remove_file(&mut self, ino: u64, rm_file_time: &SystemTime) -> Result<(), &str> {
        let inode = match self.tmp_file_map.get(&ino) {
            Some(t) => t,
            None => {
                error!(
                    "[RemoteFileManager][remove_file]file not found, ino: {}, rm_file_time: {:?}",
                    ino, rm_file_time
                );
                return Err("file not found");
            }
        };
        match self.tmp_file_trait.remove_file(inode, rm_file_time) {
            Ok(_) => {}
            Err(e) => {
                error!("[RemoteFileManager][remove_file] failed: {}", e);
                return Err("remove file failed");
            }
        };
        self.tmp_file_map.remove(&ino);
        Ok(())
    }

    pub fn mk_dir(&mut self, attr: &InodeAttributes, path: String) -> Result<Inode, &str> {
        let inode = TmpFile {
            file_name: attr.name.clone(),
            path,
            lock: RwLock::new(()),
        };
        let dir = match self
            .tmp_file_trait
            .make_dir(&inode, attr.permissions.into())
        {
            Ok(d) => d,
            Err(e) => {
                error!("[RemoteFileManager][mk_dir] failed: {}", e);
                return Err("make dir failed");
            }
        };
        self.tmp_file_map.insert(dir.ino(), inode);
        Ok(dir)
    }

    pub fn remove_dir(&mut self, ino: u64, rm_dir_time: &SystemTime) -> Result<(), &str> {
        let inode = match self.tmp_file_map.get(&ino) {
            Some(t) => t,
            None => {
                error!(
                    "[RemoteFileManager][remove_dir]file not found, ino: {}, rm_dir_time: {:?}",
                    ino, rm_dir_time
                );
                return Err("file not found");
            }
        };
        match self.tmp_file_trait.remove_dir(inode, rm_dir_time) {
            Ok(_) => {}
            Err(e) => {
                error!("[RemoteFileManager][remove_dir] failed: {}", e);
                return Err("remove dir failed");
            }
        };
        self.tmp_file_map.remove(&ino);
        Ok(())
    }

    pub fn initialize_fs(
        &mut self,
        inodes: &mut HashMap<u64, Inode>,
        source_dir: String,
    ) -> Result<(), RemoteFileInitializeError> {
        let init_fs = std::mem::replace(&mut self.init_fs, Box::new(|_, _, _| Ok(())));
        init_fs(self, inodes, source_dir)
    }
}
