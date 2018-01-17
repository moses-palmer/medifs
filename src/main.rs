extern crate fuse;
extern crate fuse_mt;

use std::path::Path;
use std::process;

mod files;


fn main() {
    let path = Path::new(".");
    let mediafs = files::MediaFS::new();

    if let Err(_) = fuse_mt::mount(
        fuse_mt::FuseMT::new(mediafs, 1),
        &path,
        &[],
    )
    {
        println!("Failed to mount media file system");
        process::exit(1);
    }
}
