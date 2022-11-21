use std::{iter, collections::BTreeMap};

use log::debug; // Used to print messages on terminal directly

// use libc::{....};
use fuse::{FileAttr, FileType, Filesystem};
use time::Timespec; // This library is used to get system-time

#[derive(Debug, Clone, Default)]
pub struct File {
    pub data: Vec<u8>, // had to change this because append_data is a vector of 8-bit unsigned int
}

impl File {

    /* Creates a new attribute for File*/
    pub fn new_file() -> File {
        File{data: Vec::new()}
    }

    /* Returns the number of bytes of data in the file*/
    pub fn get_file_size(&self) -> u64 {
        self.data.len() as u64 // Returning a unsigned 64 integer because that's what the FileAttr needs for size
    }

    /* Appends the file with new data at a specific offset*/
    pub fn update_file(&mut self, offset: i64, append_data: &[u8]) -> () {
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
        debug!("The length of new data is {}, the total size of file is {}, and the offset was {}", append_data.len(), self.get_file_size(), offset)
    }
}

#[derive(Debug, Clone)]
pub struct Inode {
    pub name: String,
    pub nodes: BTreeMap<String, u64>, // Binary-tree for all the successive directories/files
    pub root: u64, // Represents the Inode number for parent directory

    // Most of the Inode attributes mentioned in 'fs.h' would be handled by fuse::FileAttr
}

impl Inode {
    pub fn new_inode(label: String, parent: u64) -> Inode {
        Inode{name: label, nodes: BTreeMap::new(), root: parent,}
    }
    // Will add additional functionality when needed for the full file-system
}

pub struct RamFS {
    pub files: BTreeMap<u64, File>,
    pub attrs: BTreeMap<u64, FileAttr>,
    pub inodes: BTreeMap<u64, Inode>,
    pub next_inode: u64,
}

impl RamFS {
    pub fn new() -> RamFS {
        let files_stub = BTreeMap::new(); 
        let root = Inode::new("/".to_string(), 1 as u64); // Setting '/' as root and assigning Inode value for it to 1
        
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
            kind: Directory, //FileType,
            perm: 0o755, //u16, The initial permission for a directory according to ramFS in Linux
            nlink: 0, //u32,
            uid: 0, //u32,
            gid: 0, //u32,
            rdev: 0, //u32,
            flags: 0, //u32,
        }
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

    // Other functions necessary for managing a File-system
    fn get_next_ino(&mut self) -> u64 { // This is function is straight-up from ramFS in linux
        self.next_inode += 1;
        self.next_inode
    }

    // Out of all the function that the fuse::FileSystem implements there are handful of them which need tweaking
    pub fn getattr
}
