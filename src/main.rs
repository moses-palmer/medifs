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

use std::process;

mod data;
mod files;
mod types;


fn main() {
    let matches = clap::App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            clap::Arg::with_name("MOUNT_POINT")
                .help("The target mount point.")
                .required(true),
        )
        .get_matches();

    let mount_point = matches
        .value_of("MOUNT_POINT")
        .unwrap()
        .parse::<types::MountPoint>()
        .unwrap_or_else(|e| { e.exit(); });
    let mediafs = files::MediaFS::new("All".into());

    if let Err(e) = fuse_mt::mount(
        fuse_mt::FuseMT::new(mediafs, 1),
        &mount_point,
        &[],
    )
    {
        println!("Failed to mount media file system: {}", e);
        process::exit(1);
    }
}
