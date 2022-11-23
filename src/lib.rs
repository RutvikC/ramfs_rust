extern crate fuse;
extern crate libc;
extern crate time;
#[macro_use]
extern crate log;
extern crate env_logger;

use std::iter;
use std::collections::BTreeMap;
//use log::debug; // Used to print messages on terminal directly
use std::ffi::OsStr;
use libc::{ENOENT, EINVAL, EEXIST, ENOTEMPTY};
use fuse::{FileAttr, FileType, Filesystem, Request,
    ReplyAttr, ReplyData, ReplyEntry, ReplyDirectory,
    ReplyEmpty, ReplyWrite, ReplyOpen, ReplyCreate};
use time::Timespec; // This library is used to get system-time


#[derive(Debug, Clone, Default)]
pub struct File {
    data: Vec<u8>, // had to change this because append_data is a vector of 8-bit unsigned int
}

impl File {
    /* Creates a new attribute for File*/
    fn new_file() -> File {
        File{data: Vec::new()}
    }

    /* Returns the number of bytes of data in the file*/
    fn get_file_size(&self) -> u64 {
        self.data.len() as u64 // Returning a unsigned 64 integer because that's what the FileAttr needs for size
    }

    /* Appends the file with new data at a specific offset*/
    fn update_file(&mut self, offset: i64, append_data: &[u8]) -> u64 {
        let offset: usize = offset as usize;

        if offset >= self.data.len() {
            self.data.extend(iter::repeat(0).take(offset - self.data.len())); // Extending with 0s until we reach the offset bit
            self.data.splice(offset.., append_data.iter().cloned()); // Basically we are appending the new data after the offset as there is no data after that
        }
        else if offset + append_data.len() > self.data.len() {
            self.data.splice(offset.., append_data.iter().cloned()); // Same as before
        }
        else {
            self.data.splice(offset..offset, append_data.iter().cloned()); // Here we want to append data right after the offset
        }
        append_data.len() as u64
    }

    /* Shortens the vector, keeping the first 'size' elements and dropping the rest. */
    fn truncate_bytes(&mut self, size: u64) {
        self.data.truncate(size as usize);
    }
}

#[derive(Debug, Clone)]
pub struct Inode {
    name: String,
    nodes: BTreeMap<String, u64>, // Binary-tree for all the successive directories/files
    root: u64, // Represents the Inode number for parent directory

    // Most of the Inode attributes mentioned in 'fs.h' would be handled by fuse::FileAttr
}

impl Inode {
    fn new_inode(label: String, parent: u64) -> Inode {
        Inode{name: label, nodes: BTreeMap::new(), root: parent,}
    }
}

pub struct RamFS {
    files: BTreeMap<u64, File>,
    attrs: BTreeMap<u64, FileAttr>,
    inodes: BTreeMap<u64, Inode>,
    next_inode: u64,
}

impl RamFS {
    pub fn new() -> RamFS {
        let files_stub = BTreeMap::new(); 
        let root = Inode::new_inode("/".to_string(), 1 as u64); // Setting '/' as root and assigning Inode value for it to 1
        
        let ts = time::now().to_timespec();
        let mut root_dir_attrs = BTreeMap::new();
        let attr = FileAttr { // Defining attributes for root directory
            ino: 1, //u64 Since the inode-value of our root directory is 1
            size: 0, //u64,
            blocks: 0, //u64,
            atime: ts, //Timespec,
            mtime: ts, //Timespec,
            ctime: ts, //Timespec,
            crtime: ts, //Timespec,
            kind: FileType::Directory, //FileType,
            perm: 0o755, //u16, The initial permission for a directory according to ramFS in Linux
            nlink: 0, //u32,
            uid: 0, //u32,
            gid: 0, //u32,
            rdev: 0, //u32,
            flags: 0, //u32,
        };
        root_dir_attrs.insert(1, attr); // Updating root-directory attributes

        let mut root_dir_inode = BTreeMap::new();
        root_dir_inode.insert(1, root); // Adding the root directory inode in the FS

        RamFS {
            files: files_stub,
            attrs: root_dir_attrs,
            inodes: root_dir_inode,
            next_inode: 2, // Moving in order after creating the initial root directory
        }
    }

    /* Returns the next inode value in filesystem tree*/
    fn get_next_inode(&mut self) -> u64 { // This is function is straight-up from ramFS in linux
        self.next_inode += 1;
        self.next_inode
    }
}

// Out of all the function that the fuse::FileSystem implements there are handful of them which need tweaking
impl Filesystem for RamFS {

    /* This function gets the file's attributes for specified 'ino' value */
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        match self.attrs.get_mut(&ino) {
            Some(attr) => {
                reply.attr(&Timespec::new(1,0), attr);
            }
            // If no matching inode value found, then throw error
            None => {
                error!("getattr: Cannot find inode: {}", ino);
                reply.error(ENOENT)
            },
        };
    }

    /* This function updates the FileType at 'ino' attributes */
    // There are only a handful of attributes that can actually be changed once a FileType is instantiated
    fn setattr(&mut self, _req: &Request, ino: u64, _mode: Option<u32>, uid: Option<u32>, gid: Option<u32>, size: Option<u64>, atime: Option<Timespec>, mtime: Option<Timespec>, _fh: Option<u64>, crtime: Option<Timespec>, _chgtime: Option<Timespec>, _bkuptime: Option<Timespec>, _flags: Option<u32>, reply: ReplyAttr) {
        match self.attrs.get_mut(&ino) {
            // After getting the matched ino FileType, update the new attribute values
            Some(attr) => {
                match atime {
                    Some(new_atime) => attr.atime = new_atime,
                    None => {}
                }
                match mtime {
                    Some(new_mtime) => attr.mtime = new_mtime,
                    None => {}
                }
                match crtime {
                    Some(new_crtime) => attr.crtime = new_crtime,
                    None => {}
                }
                match uid {
                    Some(new_uid) => {
                        attr.uid = new_uid;
                    }
                    None => {}
                }
                match gid {
                    Some(new_gid) => {
                        attr.gid = new_gid;
                    }
                    None => {}
                }
                match size {
                    Some(new_size) => {
                        if let Some(memfile) = self.files.get_mut(&ino) {
                            // First actually update the bytes in the file and then update the attr value
                            memfile.truncate_bytes(new_size);
                            attr.size = new_size;
                        }
                    }
                    None => {}
                }
                reply.attr(&Timespec::new(1,0), attr);
            }
            None => {
                error!("setattr: Cannot find inode: {}", ino);
                reply.error(ENOENT);
            }
        }
    }

    fn readdir(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, mut reply: ReplyDirectory) {
        let mut entries = vec![];
        entries.push((ino, FileType::Directory, "."));
        if let Some(inode) = self.inodes.get(&ino) {
            entries.push((inode.root, FileType::Directory, ".."));
            for (child, child_ino) in &inode.nodes {
                let child_attrs = &self.attrs.get(child_ino).unwrap();
                entries.push((child_attrs.ino, child_attrs.kind, &child));
            }

            if entries.len() > 0 {
                // Offset of 0 means no offset.
                // Non-zero offset means the passed offset has already been seen, and we should start after
                // it.
                let to_skip = if offset == 0 { offset } else { offset + 1 } as usize;
                for (i, entry) in entries.into_iter().enumerate().skip(to_skip) {
                    reply.add(entry.0, i as i64, entry.1, entry.2);
                }
            }
            reply.ok();
        } else {
            error!("readdir: cannot find inode: {} in filesystem's inodes", ino);
            reply.error(ENOENT)
        }
    }

    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        match self.inodes.get(&parent) {
            Some(parent_ino) => {
                let inode = match parent_ino.nodes.get(name.to_str().unwrap()) {
                    Some(inode) => inode,
                    None => {
                        error!("lookup: {} is not in parent's {} children", name.to_str().unwrap(), parent);
                        reply.error(ENOENT);
                        return;
                    }
                };
                match self.attrs.get(inode) {
                    Some(attr) => {
                        reply.entry(&Timespec::new(1,0), attr, 0);
                    }
                    None => {
                        error!("lookup: inode {} is not in filesystem's attributes", inode);
                        reply.error(ENOENT);
                    }
                };
            },
            None => {
                error!("lookup: parent inode {} is not in filesystem's attributes", parent);
                reply.error(ENOENT);
            }
        };
    }

    fn rmdir(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let mut rmdir_ino = 0;
        if let Some(parent_ino) = self.inodes.get_mut(&parent) {
            match parent_ino.nodes.get(&name.to_str().unwrap().to_string()) {
                Some(dir_ino) => {
                    rmdir_ino = *dir_ino;
                }
                None => {
                    error!("rmdir: {} is not in parent's {} children", name.to_str().unwrap(), parent);
                    reply.error(ENOENT);
                    return;
                }
            }
        }
        if let Some(dir) = self.inodes.get_mut(&rmdir_ino) {
            if dir.nodes.is_empty() {
                self.attrs.remove(&rmdir_ino);
            } else {
                reply.error(ENOTEMPTY);
                return;
            }
        }
        if let Some(parent_ino) = self.inodes.get_mut(&parent) {
            parent_ino.nodes.remove(&name.to_str().unwrap().to_string());
        }
        self.inodes.remove(&rmdir_ino);
        reply.ok();
    }

    fn mkdir(&mut self, _req: &Request, parent: u64, name: &OsStr, _mode: u32, reply: ReplyEntry) {
        let ts = time::now().to_timespec();
        let attr = FileAttr {
            ino: self.get_next_inode(),
            size: 0,
            blocks: 0,
            atime: ts,
            mtime: ts,
            ctime: ts,
            crtime: ts,
            kind: FileType::Directory,
            perm: 0o644,
            nlink: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
        };

        if let Some(parent_ino) = self.inodes.get_mut(&parent) {
            if parent_ino.nodes.contains_key(name.to_str().unwrap()) {
                reply.error(EEXIST);
                return;
            }
            parent_ino.nodes.insert(name.to_str().unwrap().to_string(), attr.ino);
            self.attrs.insert(attr.ino, attr);
        } else {
            error!("mkdir: parent {} is not in filesystem inodes", parent);
            reply.error(EINVAL);
            return;
        }
        self.inodes.insert(attr.ino, Inode::new_inode(name.to_str().unwrap().to_string(), parent));
        reply.entry(&Timespec::new(1,0), &attr, 0)
    }

    fn open(&mut self, _req: &Request, _ino: u64, _flags: u32, reply: ReplyOpen) {
        reply.opened(0, 0);
    }

    fn unlink(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEmpty) {
        let mut old_ino = 0;
        if let Some(parent_ino) = self.inodes.get_mut(&parent) {
            match parent_ino.nodes.remove(&name.to_str().unwrap().to_string()) {
                Some(ino) => {
                    match self.attrs.remove(&ino) {
                        Some(attr) => {
                            if attr.kind == FileType::RegularFile{
                                self.files.remove(&ino);
                            }
                            old_ino = ino;
                        },
                        None => {
                            old_ino = ino;
                        },
                    }
                }
                None => {
                    error!("unlink: {} is not in parent's {} children", name.to_str().unwrap(), parent);
                    reply.error(ENOENT);
                    return;
                }
            }
        };
        self.inodes.remove(&old_ino);
        reply.ok();
    }

    fn create(&mut self, _req: &Request, parent: u64, name: &OsStr, _mode: u32, _flags: u32, reply: ReplyCreate) {
        let new_ino = self.get_next_inode();
        match self.inodes.get_mut(&parent) {
            Some(parent_ino) => {
                if let Some(ino) = parent_ino.nodes.get_mut(&name.to_str().unwrap().to_string()) {
                    reply.created(&Timespec::new(1,0), self.attrs.get(&ino).unwrap(), 0, 0 ,0);
                    return;
                } 
                else {
                    let ts = time::now().to_timespec();
                    let attr = FileAttr {
                        ino: new_ino,
                        size: 0,
                        blocks: 0,
                        atime: ts,
                        mtime: ts,
                        ctime: ts,
                        crtime: ts,
                        kind: FileType::RegularFile,
                        perm: 0o755,
                        nlink: 0,
                        uid: 0,
                        gid: 0,
                        rdev: 0,
                        flags: 0,
                    };
                    self.attrs.insert(attr.ino, attr);
                    self.files.insert(attr.ino, File::new_file());
                    reply.created(&Timespec::new(1,0), &attr, 0, 0, 0);
                }
                parent_ino.nodes.insert(name.to_str().unwrap().to_string(), new_ino);
            }
            None => {
                error!("create: parent {} is not in filesystem's inodes", parent);
                reply.error(EINVAL);
                return;
            }
        }
        self.inodes.insert(new_ino, Inode::new_inode(name.to_str().unwrap().to_string(), parent));
    }

    fn write(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, data: &[u8], _flags: u32, reply: ReplyWrite) {
        let ts = time::now().to_timespec();
        match self.files.get_mut(&ino) {
            Some(fp) => {
                match self.attrs.get_mut(&ino) {
                    Some(attr) => {
                        let size = fp.update_file(offset, &data);
                        attr.atime = ts;
                        attr.mtime = ts;
                        attr.size = fp.get_file_size();
                        reply.written(size as u32);
                    }
                    None => {
                        error!("write: ino {} is not in filesystem's attributes", ino);
                        reply.error(ENOENT);
                    }
                }
            }
            None => reply.error(ENOENT),
        }
    }

    fn read(&mut self, _req: &Request, ino: u64, _fh: u64, offset: i64, _size: u32, reply: ReplyData) {   
        match self.files.get_mut(&ino) {
            Some(fp) => {
                let mut thread_attrs = self.attrs.clone();
                let thread_fp = fp.clone();
                match thread_attrs.get_mut(&ino) {
                    Some(attr) => {
                        attr.atime = time::now().to_timespec();
                        reply.data(&thread_fp.data[offset as usize..]);
                    },
                    None => {
                        error!("read: ino {} is not in filesystem's attributes", ino);
                        reply.error(ENOENT);
                    },
                }
            }
            None => {
                reply.error(ENOENT);
            }
        }
    }

    fn rename(&mut self, _req: &Request, parent: u64, name: &OsStr, newparent: u64, newname: &OsStr, reply: ReplyEmpty) {
        if self.inodes.contains_key(&parent) && self.inodes.contains_key(&newparent) {
            let file_ino;
            match self.inodes.get_mut(&parent) {
                Some(parent_ino) => {
                    if let Some(ino) = parent_ino.nodes.remove(&name.to_str().unwrap().to_string()) {
                        file_ino = ino;
                    } else {
                        error!("{} not found in parent {}", name.to_str().unwrap().to_string(), parent);
                        reply.error(ENOENT);
                        return;
                    }
                }
                None => {
                    error!("rename: parent {} is not in filesystem inodes", parent);
                    reply.error(EINVAL);
                    return;
                }
            }
            if let Some(newparent_ino) = self.inodes.get_mut(&newparent) {
                newparent_ino.nodes.insert(newname.to_str().unwrap().to_string(), file_ino);

            }
        }
        reply.ok();
    }
}
