#[macro_use]
extern crate clap;
extern crate fuse;
extern crate fuse_mt;
extern crate libc;
extern crate mime;
extern crate mime_guess;
extern crate time;

#[cfg(test)]
extern crate tempdir;

use std::path::Path;
use std::process;

mod data;
mod files;


fn main() {
    let path = Path::new(".");
    let mediafs = files::MediaFS::new("All".into());

    if let Err(e) = fuse_mt::mount(
        fuse_mt::FuseMT::new(mediafs, 1),
        &path,
        &[],
    )
    {
        println!("Failed to mount media file system: {}", e);
        process::exit(1);
    }
}
