use std::{
    collections::HashMap,
    ffi::OsStr,
    time::{Duration, SystemTime},
};

use fuser::{
    consts::FOPEN_DIRECT_IO, FileType, Filesystem, ReplyAttr, ReplyCreate, ReplyData,
    ReplyDirectory, ReplyEntry, ReplyOpen, Request, TimeOrNow,
};
use libc::ENOENT;
use log::{debug, info};

use crate::{
    common::{FMODE_EXEC, MAX_NAME_LENGTH, RFUSE_S_ISVTX},
    inode::{Inode, InodeAttributes, InodeKind},
    remote_fs::{InitFsFuncType, RemoteFileManager},
    tmp_file::TmpFileTrait,
    utils::check_access,
};

pub enum RFuseFSOP {
    ReInItFs,
    Exit,
    Nothing,
}

pub struct RFuseFS {
    fs_name: String, // 后期可能会有多个目录的需求，这个先保留
    source_dir: String,
    inodes: HashMap<u64, Inode>,
    direct_io: bool,
    remote_file_manager: RemoteFileManager,
}

impl RFuseFS {
    pub fn new(
        fs_name: String,
        direct_io: bool,
        source_dir: String,
        init_fs_func: Box<InitFsFuncType>,
        tmp_file_trait: Box<dyn TmpFileTrait + 'static + Send>,
    ) -> Self {
        Self {
            fs_name,
            source_dir,
            inodes: HashMap::new(),
            direct_io,
            remote_file_manager: RemoteFileManager::new(init_fs_func, tmp_file_trait),
        }
    }

    pub fn lookup_name(&self, parent: u64, name: &str) -> Option<u64> {
        let parent_inode = self.inodes.get(&parent).unwrap();
        for ino in parent_inode.children_ino.iter() {
            let inode = self.inodes.get(ino).unwrap();
            if inode.attr.name.eq(name) {
                return Some(*ino);
            }
        }
        None
    }

    /// 清空inode, 重建
    pub fn clean_inode(&mut self) {
        self.inodes.clear();
    }

    pub fn get_inode(&self, inode: u64) -> Option<&Inode> {
        self.inodes.get(&inode)
    }

    pub fn write_inode(&mut self, inode: &Inode) {
        self.inodes.insert(inode.ino, inode.clone());
    }

    pub fn re_init_fs(&mut self) {
        self.clean_inode();
        match self
            .remote_file_manager
            .initialize_fs(&mut self.inodes, self.source_dir.clone())
        {
            Ok(_) => {}
            Err(e) => {
                debug!("[RFuseFS][init] -> Initialize filesystem. {}", e);
            }
        };
    }
}

impl Filesystem for RFuseFS {
    fn init(
        &mut self,
        _req: &Request,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), libc::c_int> {
        info!("[RFuseFS][init] -> Initialize filesystem.");

        match self
            .remote_file_manager
            .initialize_fs(&mut self.inodes, self.source_dir.clone())
        {
            Ok(_) => {}
            Err(e) => {
                debug!("[RFuseFS][init] -> Initialize filesystem. {}", e);
                return Err(libc::EIO);
            }
        };
        // debug!("Inodes: {:?}", self.inodes);
        Ok(())
    }

    fn lookup(&mut self, req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        // info!("[RFuseFS][lookup] -> Look up a directory entry by name.");
        if name.len() > MAX_NAME_LENGTH as usize {
            reply.error(libc::ENAMETOOLONG);
            return;
        }

        let parent_inode = self.get_inode(parent).unwrap();
        if !check_access(
            parent_inode.attr.uid,
            parent_inode.attr.gid,
            parent_inode.attr.permissions,
            req.uid(),
            req.gid(),
            libc::X_OK,
        ) {
            reply.error(libc::EACCES);
            return;
        }

        let name = name.to_str().unwrap().to_owned();
        // debug!(
        //     "[RFuseFS][lookup] -> Look up a directory entry and get its attributes. {}",
        //     name.clone()
        // );
        match self.lookup_name(parent, &name) {
            Some(ino) => {
                let inode = self.get_inode(ino).unwrap();
                reply.entry(&Duration::new(0, 0), &inode.file_attr(), 0)
            }
            None => reply.error(ENOENT),
        }
    }

    fn getattr(&mut self, _req: &Request, ino: u64, _fh: Option<u64>, reply: ReplyAttr) {
        // info!("[RFuseFS][getattr] -> Get attributes of a file.");
        match self.get_inode(ino) {
            Some(inode) => {
                // debug!(
                //     "[RFuseFS][getattr] -> Get file attributes. {}",
                //     inode.attr.name.clone()
                // );
                reply.attr(&Duration::new(0, 0), &inode.file_attr())
            }
            None => reply.error(ENOENT),
        }
    }

    fn setattr(
        &mut self,
        req: &Request,
        ino: u64,
        mode: Option<u32>,
        uid: Option<u32>,
        gid: Option<u32>,
        size: Option<u64>,
        _atime: Option<TimeOrNow>,
        mtime: Option<TimeOrNow>,
        ctime: Option<SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<SystemTime>,
        _chgtime: Option<SystemTime>,
        _bkuptime: Option<SystemTime>,
        _flags: Option<u32>,
        reply: ReplyAttr,
    ) {
        info!("[RFuseFS][setattr] -> Set attributes of a file.");

        let inode = self.inodes.get_mut(&ino).unwrap();

        // 确认权限
        if inode.attr.uid != req.uid()
            && !check_access(
                inode.attr.uid,
                inode.attr.gid,
                inode.attr.permissions,
                req.uid(),
                req.gid(),
                libc::W_OK,
            )
        {
            reply.error(libc::EACCES);
            return;
        }

        if let Some(mode) = mode {
            inode.attr.permissions = mode as u16;
        }
        if let Some(size) = size {
            // NOTE: 当文件大小为空时，这里会直接设置文件大小为0，不会调用write方法
            inode.attr.size = size;
        }
        if let Some(mtime) = mtime {
            inode.attr.mtime = match mtime {
                TimeOrNow::SpecificTime(time) => time,
                fuser::TimeOrNow::Now => SystemTime::now(),
            };
        }

        if let Some(uid) = uid {
            inode.attr.uid = uid;
        }

        if let Some(gid) = gid {
            inode.attr.gid = gid;
        }

        if let Some(ctime) = ctime {
            inode.attr.ctime = ctime;
        }

        // debug!(
        //     "[RFuseFS][setattr] -> Set file attributes. {}",
        //     inode.attr.name.clone()
        // );
        match self.remote_file_manager.set_attr(ino, &inode.attr) {
            Ok(_) => {
                reply.attr(&Duration::new(0, 0), &inode.file_attr());
            }
            Err(e) => {
                debug!("[RFuseFS][setattr] -> Set file attributes. {}", e);
                reply.error(libc::EIO);
            }
        };
    }

    fn open(&mut self, req: &Request<'_>, ino: u64, flags: i32, reply: ReplyOpen) {
        let (access_mask, _read, _write) = match flags & libc::O_ACCMODE {
            libc::O_RDONLY => {
                // Behavior is undefined, but most filesystems return EACCES
                if flags & libc::O_TRUNC != 0 {
                    reply.error(libc::EACCES);
                    return;
                }
                if flags & FMODE_EXEC != 0 {
                    // Open is from internal exec syscall
                    (libc::X_OK, true, false)
                } else {
                    (libc::R_OK, true, false)
                }
            }
            libc::O_WRONLY => (libc::W_OK, false, true),
            libc::O_RDWR => (libc::R_OK | libc::W_OK, true, true),
            // Exactly one access mode flag must be specified
            _ => {
                reply.error(libc::EINVAL);
                return;
            }
        };

        match self.inodes.get_mut(&ino) {
            Some(inode) => {
                debug!(
                    "[RFuseFS][open] -> Open a file. {}",
                    inode.attr.name.clone()
                );
                if check_access(
                    inode.attr.uid,
                    inode.attr.gid,
                    inode.attr.permissions,
                    req.uid(),
                    req.gid(),
                    access_mask,
                ) {
                    let open_flags = if self.direct_io { FOPEN_DIRECT_IO } else { 0 };
                    reply.opened(ino, open_flags);
                }
            }
            None => {
                reply.error(ENOENT);
            }
        };
    }

    fn read(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock: Option<u64>,
        reply: ReplyData,
    ) {
        info!("[RFuseFS][read] -> Read data from an open file.");
        assert!(offset >= 0);
        // if fh != ino {
        //     reply.error(EACCES);
        //     return;
        // }
        let file_size = match self.get_inode(ino) {
            Some(inode) => {
                debug!("[RFuseFS][read] -> Read data. {}", inode.attr.name.clone());
                inode.attr.size
            }
            None => {
                reply.error(ENOENT);
                return;
            }
        };

        let read_size = std::cmp::min(size, file_size.saturating_sub(offset as u64) as u32);
        let mut buf = vec![0; read_size as usize];
        match self
            .remote_file_manager
            .read(ino, &mut buf, read_size as u64)
        {
            Ok(_) => {}
            Err(e) => {
                debug!("[RFuseFS][read] -> Read data. {}", e);
                reply.error(libc::EIO);
                return;
            }
        };

        reply.data(&buf);
    }

    fn readdir(
        &mut self,
        _req: &Request,
        ino: u64,
        _fh: u64,
        offset: i64,
        mut reply: ReplyDirectory,
    ) {
        // info!("[RFuseFS][readdir] -> Read directory entries.");
        let inode = self.get_inode(ino).unwrap();
        let mut entires = vec![
            (ino, FileType::Directory, ".".to_owned()),
            (ino, FileType::Directory, "..".to_owned()),
        ];

        let children: Vec<(u64, FileType, String)> = inode
            .children_ino
            .clone()
            .iter()
            .map(|ino| {
                let inode = self.get_inode(*ino).unwrap();
                (inode.ino, inode.attr.kind.into(), inode.attr.name.clone())
            })
            .collect();

        entires.extend(children);

        for (index, (ino, kind, name)) in entires.into_iter().enumerate().skip(offset as usize) {
            if reply.add(ino, (index + 1) as i64, kind, name) {
                break;
            }
        }

        reply.ok();
    }

    fn rmdir(&mut self, req: &Request<'_>, parent: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
        info!("[RFuseFS][rmdir] -> Remove a directory.");
        let name = name.to_str().unwrap().to_string();
        let ino = match self.lookup_name(parent, &name) {
            Some(ino) => ino,
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        // 做一些异常处理
        let inode = self.get_inode(ino).unwrap();
        // 不是文件夹类型
        if !inode.is_dir() {
            reply.error(libc::ENOTDIR);
            return;
        }
        // 文件夹不为空
        if !inode.children_ino.is_empty() {
            reply.error(libc::ENOTEMPTY);
            return;
        }
        let mut parent_inode = self.get_inode(parent).unwrap().clone();

        // 确认是否有当前文件夹权限
        if !check_access(
            parent_inode.attr.uid,
            parent_inode.attr.gid,
            parent_inode.attr.permissions,
            req.uid(),
            req.gid(),
            libc::W_OK,
        ) {
            reply.error(libc::EACCES);
            return;
        }

        // "Sticky bit" handling
        if parent_inode.attr.permissions & RFUSE_S_ISVTX != 0
            && req.uid() != 0
            && req.uid() != parent_inode.attr.uid
            && req.uid() != inode.attr.uid
        {
            reply.error(libc::EACCES);
            return;
        }

        let new_time = SystemTime::now();
        parent_inode.remove_child(ino);
        match self.remote_file_manager.remove_dir(ino, &new_time) {
            Ok(_) => {}
            Err(e) => {
                debug!("[RFuseFS][rmdir] -> Remove a directory. {}", e);
                reply.error(libc::EIO);
                return;
            }
        };
        parent_inode.attr.mtime = new_time;
        parent_inode.attr.ctime = new_time;
        self.write_inode(&parent_inode);
        self.inodes.remove(&ino);
        reply.ok();
    }

    fn mkdir(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        _umask: u32,
        reply: ReplyEntry,
    ) {
        info!("[RFuseFS][mkdir] -> Create a directory.");
        let name = name.to_str().unwrap().to_string();
        if self.lookup_name(parent, &name).is_some() {
            reply.error(libc::EEXIST);
            return;
        }

        let parent_inode = self.inodes.get_mut(&parent).unwrap();

        if !check_access(
            parent_inode.attr.uid,
            parent_inode.attr.gid,
            parent_inode.attr.permissions,
            req.uid(),
            req.gid(),
            libc::W_OK,
        ) {
            reply.error(libc::EACCES);
            return;
        }

        parent_inode.attr.mtime = SystemTime::now();
        assert!(parent_inode.is_dir());

        let new_path: String = {
            if parent_inode.attr.name.is_empty() {
                parent_inode.attr.path.clone()
            } else {
                parent_inode.attr.path.clone() + &parent_inode.attr.name + "/"
            }
        };

        let mut attr = InodeAttributes::new(name.clone(), InodeKind::Directory, new_path.clone());

        attr.permissions = mode as u16;

        let mut new_inode = Inode::new(parent, attr.clone());
        let new_file_meta = match self
            .remote_file_manager
            .mk_dir(&attr, self.source_dir.clone() + &attr.path)
        {
            Ok(meta) => meta,
            Err(e) => {
                debug!("[RFuseFS][mkdir] -> Create a directory. {}", e);
                reply.error(libc::EIO);
                return;
            }
        };
        // 这里直接回写了inode的信息
        new_inode.parent_ino = parent;
        new_inode.ino = new_file_meta.ino();
        new_inode.attr = new_file_meta.attr;
        new_inode.attr.path = new_path;

        debug!(
            "[RFuseFS][mkdir] -> Create a directory. {}",
            new_inode.attr.path.clone() + &new_inode.attr.name
        );

        parent_inode.insert_child(new_inode.ino);
        self.inodes.insert(new_inode.ino, new_inode.clone());
        reply.entry(&Duration::new(0, 0), &new_inode.file_attr(), 0);
    }

    fn rename(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        #[cfg(target_os = "linux")] flags: u32,
        #[cfg(not(target_os = "linux"))] _flags: u32,
        reply: fuser::ReplyEmpty,
    ) {
        debug!("[RFuseFS][rename] -> Rename a file.");

        // 旧文件的inode
        let mut inode = match self.lookup_name(parent, name.to_str().unwrap()) {
            Some(ino) => self.get_inode(ino).unwrap().clone(),
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        // 新文件的inode
        let mut parent_inode = match self.get_inode(parent) {
            Some(ino) => ino.clone(),
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        // 确认是否有当前文件夹权限
        if !check_access(
            parent_inode.attr.uid,
            parent_inode.attr.gid,
            parent_inode.attr.permissions,
            req.uid(),
            req.gid(),
            libc::W_OK,
        ) {
            reply.error(libc::EACCES);
            return;
        }

        #[cfg(target_os = "linux")]
        debug!(
            "[RFuseFS][rename] -> flags: {}",
            flags & libc::RENAME_EXCHANGE
        );

        // "Sticky bit" handling
        if parent_inode.attr.permissions & RFUSE_S_ISVTX != 0
            && req.uid() != 0
            && req.uid() != parent_inode.attr.uid
            && req.uid() != inode.attr.uid
        {
            reply.error(libc::EACCES);
            return;
        }

        let new_name = newname.to_str().unwrap().to_string();
        if self.lookup_name(parent, &new_name).is_some() {
            reply.error(libc::EEXIST);
            return;
        }

        // 新的父文件夹
        let mut new_parent_inode = match self.get_inode(newparent) {
            Some(inode) => inode.clone(),
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        debug!(
            "[RFuseFS][rename] -> Rename a file. name: {} ",
            inode.attr.name,
        );

        debug!(
            "[RFuseFS][rename] -> Rename a file. {} -> {}",
            inode.attr.path.clone() + &inode.attr.name,
            new_parent_inode.attr.path.clone() + &new_parent_inode.attr.name + &new_name
        );

        // 确认是否有新的文件夹的权限
        if !check_access(
            new_parent_inode.attr.uid,
            new_parent_inode.attr.gid,
            new_parent_inode.attr.permissions,
            req.uid(),
            req.gid(),
            libc::W_OK,
        ) {
            reply.error(libc::EACCES);
            return;
        }

        // "Sticky bit" handling in new_parent
        if new_parent_inode.attr.permissions & RFUSE_S_ISVTX != 0 {
            if let Some(existing_attrs) = self.lookup_name(newparent, newname.to_str().unwrap()) {
                let existing_inode = self.get_inode(existing_attrs).unwrap();
                if req.uid() != 0
                    && req.uid() != new_parent_inode.attr.uid
                    && req.uid() != existing_inode.attr.uid
                {
                    reply.error(libc::EACCES);
                    return;
                }
            }
        }

        // TODO: 后续可以支持一下这个用的不多，它是用来交换两个文件夹名字的
        // https://github.com/cberner/fuser/blob/99aa528056f02cbcfc5283a16c6f05643f536271/examples/simple.rs#L1172-L1220
        #[cfg(target_os = "linux")]
        if flags & libc::RENAME_EXCHANGE != 0 {
            reply.error(libc::ENOSYS);
            return;
        }

        // 这里好像是不为空的文件夹不允许改名
        // Only overwrite an existing directory if it's empty
        if let Some(new_name_attrs) = self.lookup_name(newparent, newname.to_str().unwrap()) {
            let existing_inode = self.get_inode(new_name_attrs).unwrap();
            if existing_inode.is_dir() && !existing_inode.children_ino.is_empty() {
                reply.error(libc::ENOTEMPTY);
                return;
            }
        }

        // Only move an existing directory to a new parent, if we have write access to it,
        // because that will change the ".." link in it
        if inode.is_dir()
            && parent != newparent
            && !check_access(
                inode.attr.uid,
                inode.attr.gid,
                inode.attr.permissions,
                req.uid(),
                req.gid(),
                libc::W_OK,
            )
        {
            reply.error(libc::EACCES);
            return;
        }

        // 真正修改文件
        let new_time = SystemTime::now();
        let new_path = {
            if new_parent_inode.attr.name.is_empty() {
                new_parent_inode.attr.path.clone()
            } else {
                new_parent_inode.attr.path.clone() + &new_parent_inode.attr.name + "/"
            }
        };
        match self.remote_file_manager.rename(
            inode.ino,
            new_name.clone(),
            self.source_dir.clone() + &new_path,
            &new_time,
        ) {
            Ok(_) => {}
            Err(e) => {
                debug!("[RFuseFS][rename] -> Rename a file. {}", e);
                reply.error(libc::EIO);
                return;
            }
        };

        // 修改为新的名字
        inode.attr.name = new_name;
        inode.attr.path = new_path;
        // 修改时间
        inode.attr.mtime = new_time;
        // inode.attr.ctime = new_time;
        // 回写到缓存
        self.write_inode(&inode);
        // 删除旧文件夹的内容
        parent_inode.remove_child(inode.ino);
        parent_inode.attr.mtime = new_time;
        // parent_inode.attr.ctime = new_time;
        self.write_inode(&parent_inode);
        // 更新新文件夹的内容
        new_parent_inode.children_ino.push(inode.ino);
        new_parent_inode.attr.mtime = new_time;
        // new_parent_inode.attr.ctime = new_time;
        self.write_inode(&new_parent_inode);

        reply.ok();
    }

    fn write(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        data: &[u8],
        write_flags: u32,
        flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyWrite,
    ) {
        info!(
            "[RFuseFS][write] -> Write data to an open file. ino: {}",
            ino
        );
        assert!(offset >= 0);

        // fio 测试不要开这个，这个只能测
        // debug!(
        //     "[RFuseFS][write] -> Write data. ino: {}, offset: {}, data: {:?}",
        //     ino,
        //     offset,
        //     String::from_utf8(data.to_vec()).unwrap()
        // );

        debug!(
            "[RFuseFS][write] -> Write data. write_flags: {}, flags: {}, ino: {}",
            write_flags, flags, ino
        );

        let inode = match self.inodes.get_mut(&ino) {
            Some(inode) => {
                debug!("write() -> Write data. {}", inode.attr.name.clone());
                inode
            }
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        let write_time = SystemTime::now();
        match self.remote_file_manager.write_file(ino, data, &write_time) {
            Ok(_) => {}
            Err(e) => {
                debug!("[RFuseFS][write] -> Write data. {}", e);
                reply.error(libc::EIO);
                return;
            }
        };
        inode.attr.mtime = write_time;
        inode.attr.ctime = write_time;
        // if data.len() + offset as usize > inode.attr.size as usize {
        //     inode.attr.size = (data.len() + offset as usize) as u64;
        // }
        inode.attr.size = data.len() as u64;
        reply.written(data.len() as u32);
    }

    fn create(
        &mut self,
        req: &Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        flags: i32,
        reply: ReplyCreate,
    ) {
        info!("[RFuseFS][create] -> Create and open a file.");
        let name = name.to_str().unwrap().to_string();
        debug!("[RFuseFS][create] -> Create and open a file. {}", name);
        if self.lookup_name(parent, &name).is_some() {
            reply.error(libc::EEXIST);
            return;
        }

        match flags & libc::O_ACCMODE {
            libc::O_RDONLY => (true, false),
            libc::O_WRONLY => (false, true),
            libc::O_RDWR => (true, true),
            _ => {
                reply.error(libc::EINVAL);
                return;
            }
        };

        let parent_inode = self.inodes.get_mut(&parent).unwrap();

        // 确认权限
        if !check_access(
            parent_inode.attr.uid,
            parent_inode.attr.gid,
            parent_inode.attr.permissions,
            req.uid(),
            req.gid(),
            libc::W_OK,
        ) {
            reply.error(libc::EACCES);
            return;
        }

        parent_inode.attr.mtime = SystemTime::now();
        assert!(parent_inode.is_dir());
        let path = {
            if parent_inode.attr.name.is_empty() {
                parent_inode.attr.path.clone()
            } else {
                parent_inode.attr.path.clone() + &parent_inode.attr.name + "/"
            }
        };
        // 这里的 attr 只是占位, 后续会被 RemoteFileManager 回写为真实数据
        let attr = InodeAttributes::new(name.clone(), InodeKind::File, path.clone());
        let mut new_inode = Inode::new(parent, attr.clone());
        let new_ino = new_inode.ino;
        let new_file_meta = match self.remote_file_manager.new_file(
            new_ino,
            attr.clone(),
            self.source_dir.clone() + &attr.path,
        ) {
            Ok(meta) => meta,
            Err(e) => {
                debug!("[RFuseFS][create] -> Create and open a file. {}", e);
                reply.error(libc::EIO);
                return;
            }
        };
        // 这里直接回写了inode的信息
        new_inode.parent_ino = parent;
        new_inode.ino = new_file_meta.ino();
        new_inode.attr = new_file_meta.attr;
        new_inode.attr.path = path; // 注意这里的 path 应该是自己管理的地址而不是, 信息源的地址

        parent_inode.insert_child(new_inode.ino);
        self.inodes.insert(new_inode.ino, new_inode.clone());
        reply.created(
            &Duration::new(0, 0),
            &new_inode.file_attr(),
            0,
            new_inode.ino,
            0,
        );
    }

    // 校验文件权限
    fn access(&mut self, req: &Request<'_>, ino: u64, mask: i32, reply: fuser::ReplyEmpty) {
        info!("[RFuseFS][access] -> Check file access permissions.");
        let inode = self.get_inode(ino).unwrap();
        let mode = inode.attr.permissions;
        if check_access(
            inode.attr.uid,
            inode.attr.gid,
            mode,
            req.uid(),
            req.gid(),
            mask,
        ) {
            reply.ok();
        } else {
            reply.error(libc::EACCES);
        }
    }

    // 删除文件
    fn unlink(&mut self, req: &Request<'_>, parent: u64, name: &OsStr, reply: fuser::ReplyEmpty) {
        info!("[RFuseFS][unlink] -> Remove a file.");
        let name = name.to_str().unwrap().to_string();
        let ino = match self.lookup_name(parent, &name) {
            Some(ino) => ino,
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        let parent_inode = match self.inodes.get_mut(&parent) {
            Some(ino) => ino,
            None => {
                reply.error(libc::ENOENT);
                return;
            }
        };

        // debug!(
        //     "[RFuseFS][unlink] -> Remove a file. parent_inode: {:#?}",
        //     parent_inode
        // );

        if !check_access(
            parent_inode.attr.uid,
            parent_inode.attr.gid,
            parent_inode.attr.permissions,
            req.uid(),
            req.gid(),
            libc::W_OK,
        ) {
            reply.error(libc::EACCES);
            return;
        }

        let uid = req.uid();
        // "Sticky bit" handling
        if parent_inode.attr.permissions & RFUSE_S_ISVTX != 0
            && uid != 0
            && uid != parent_inode.attr.uid
        // && uid != inode.attr.uid
        {
            reply.error(libc::EACCES);
            return;
        }

        let new_time = SystemTime::now();
        parent_inode.remove_child(ino);
        match self.remote_file_manager.remove_file(ino, &new_time) {
            Ok(_) => {}
            Err(e) => {
                debug!("[RFuseFS][unlink] -> Remove a file. {}", e);
                reply.error(libc::EIO);
                return;
            }
        };
        parent_inode.attr.mtime = new_time;
        parent_inode.attr.ctime = new_time;
        self.inodes.remove(&ino);
        reply.ok();
    }

    fn destroy(&mut self) {
        info!("[RFuseFS][destroy] -> Destroy {} filesystem.", self.fs_name);
    }
}
