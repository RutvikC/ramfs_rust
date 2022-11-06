extern crate log;
extern crate env_logger;

extern crate ramfs; // crate to call structs from lib.rs

use ramfs::{File, Inode};

fn main() {
    let mut file = File::new_file();
    println!("\nThe initial size of the file is: {}", file.get_file_size());

    let mut offset: i64 = 0;
    let mut data: Vec<u8> = Vec::new();
    data.extend([1, 2, 3, 4, 5].iter().copied());
    file.update_file(offset, &data);

    offset = 3;
    data.clear();
    data.push(6);
    file.update_file(offset, &data);

    println!("The file contains {:?}", file.data);

    println!("The size of the file after updating is: {}", file.get_file_size());

    let inode = Inode::new_inode("foo.txt".to_string(), 0); // For testing purpose we have given arbitary inode number for parent
    println!("\nThe file name is '{}' and the parent Inode number is: {}", inode.name, inode.root);
}
