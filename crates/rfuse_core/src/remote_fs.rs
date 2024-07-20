use std::{collections::HashMap, sync::RwLock};

use log::{debug, error, warn};

use crate::{
    inode::{Inode, InodeAttributes},
    tmp_file::{TmpFile, TmpFileTrait},
};

pub struct RemoteFileManager {
    pub tmp_file_map: HashMap<u64, TmpFile>,
}

impl RemoteFileManager {
    pub fn new() -> Self {
        RemoteFileManager {
            tmp_file_map: HashMap::new(),
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
        match self.tmp_file_map.get(&ino).unwrap().read_exact(buf, offset) {
            Ok(_) => {}
            Err(e) => {
                error!("[RemoteFileManager][read] failed: {}", e);
                return Err("read file failed");
            }
        };
        Ok(())
    }

    pub fn read_all(&self, ino: u64) -> Vec<u8> {
        match self.tmp_file_map.get(&ino).unwrap().read_all() {
            Ok(data) => data,
            Err(e) => {
                error!("[RemoteFileManager][read_all] failed: {}", e);
                Vec::new() // Return an empty Vec<u8> in case of error
            }
        }
    }

    pub fn write_file(&mut self, ino: u64, data: &[u8]) -> Result<(), &str> {
        let tmp = match self.tmp_file_map.get_mut(&ino) {
            Some(t) => t,
            None => {
                error!(
                    "[RemoteFileManager][write_file]file not found, ino: {}",
                    ino
                );
                return Err("file not found");
            }
        };
        match tmp.write(data) {
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
        let tmp = TmpFile {
            file_name: attr.name.clone(),
            path: path.clone(),
            lock: RwLock::new(()),
        };

        let meta = match tmp.create_file() {
            Ok(m) => m,
            Err(e) => {
                error!("[RemoteFileManager][new_file]create file failed: {}", e);
                return Err("create file failed");
            }
        };
        self.tmp_file_map.insert(meta.ino(), tmp);
        Ok(meta)
    }

    pub fn set_attr(&mut self, ino: u64, attr: &InodeAttributes) -> Result<(), &str> {
        let tmp = self.tmp_file_map.get_mut(&ino).unwrap();
        match tmp.set_attr(&attr) {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("[RemoteFileManager][set_attr] failed: {}", e);
                Err("set attr failed")
            }
        }
    }

    pub fn rename(&mut self, ino: u64, new_name: String, new_path: String) -> Result<(), &str> {
        let tmp = self.tmp_file_map.get_mut(&ino).unwrap();
        match tmp.rename(new_path.clone() + new_name.as_str()) {
            Ok(_) => {}
            Err(e) => {
                error!("[RemoteFileManager][rename] failed: {}", e);
                return Err("rename failed");
            }
        };
        tmp.file_name = new_name;
        tmp.path = new_path;
        Ok(())
    }

    pub fn remove_file(&mut self, ino: u64) -> Result<(), &str> {
        match self.tmp_file_map.get(&ino).unwrap().remove_file() {
            Ok(_) => {}
            Err(e) => {
                error!("[RemoteFileManager][remove_file] failed: {}", e);
                return Err("remove file failed");
            }
        };
        self.tmp_file_map.remove(&ino);
        Ok(())
    }

    pub fn mk_dir(
        &mut self,
        ino: u64,
        attr: &InodeAttributes,
        path: String,
    ) -> Result<Inode, &str> {
        let tmp = TmpFile {
            file_name: attr.name.clone(),
            path,
            lock: RwLock::new(()),
        };
        let dir = match tmp.make_dir(attr.permissions.into()) {
            Ok(d) => d,
            Err(e) => {
                error!("[RemoteFileManager][mk_dir] failed: {}", e);
                return Err("make dir failed");
            }
        };
        self.tmp_file_map.insert(ino, tmp);
        Ok(dir)
    }

    pub fn remove_dir(&mut self, ino: u64) -> Result<(), &str> {
        let tmp = self.tmp_file_map.get(&ino).unwrap();
        match tmp.remove_dir() {
            Ok(_) => {}
            Err(e) => {
                error!("[RemoteFileManager][remove_dir] failed: {}", e);
                return Err("remove dir failed");
            }
        };
        self.tmp_file_map.remove(&ino);
        Ok(())
    }
}

pub trait RemoteFileManagerTrait {
    // 初始化文件树
    fn init_fs(
        &mut self,
        inodes: &mut HashMap<u64, Inode>,
        source_dir: String,
    ) -> Result<(), &str> {
        warn!(
            "[RemoteFileManagerTrait][Not Implemented] init_fs(inodes: {:#?}, source_dir: {:#?})",
            inodes, source_dir
        );
        Err("init_fs failed")
    }
}

impl RemoteFileManagerTrait for RemoteFileManager {}
