use std::{iter, collections::BTreeMap};

use log::debug; // Used to print messages on terminal directly

// use libc::{....};

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

