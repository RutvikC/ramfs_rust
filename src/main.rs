// extern crates
extern crate fuse;
extern crate libc;
extern crate time;
#[macro_use]
extern crate log;
extern crate env_logger;

// Bindings
use ramfs::RamFS;
use std::env;
//use std::ffi::OsStr;

fn main() {
    // Init log level system (error, warn, info, debug, trace) for this program
    env_logger::init();

    // Create a file system instance
    let fs = RamFS::new();

    /* Extract the mountpoint from the command line argument
     * If the argument is not found generate an error and return
    */
    let mountpoint = match env::args().nth(1) {
        Some(path) => path,
        None => {
            error!("Usage: {} <mount_point>. Provide mountpoint argument", env::args().nth(0).unwrap());
            return;
        }
    };

    // Mount the file system using fuse's mount api
    fuse::mount(fs, &mountpoint, &[]).unwrap();
}