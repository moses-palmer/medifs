#[macro_use]
extern crate clap;
extern crate fuse;
extern crate fuse_mt;
extern crate libc;
extern crate mime;
extern crate mime_guess;
extern crate time;
extern crate walkdir;

#[cfg(test)]
extern crate tempdir;

use std::ffi;
use std::process;
use std::sync;

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
        .arg(
            clap::Arg::with_name("FUSE_OPTION")
                .help("Options passed to FUSE")
                .multiple(true)
                .value_delimiter(",")
                .short("o")
                .takes_value(true),
        )
        .get_matches();

    let mount_point = matches
        .value_of("MOUNT_POINT")
        .unwrap()
        .parse::<types::MountPoint>()
        .unwrap_or_else(|e| { e.exit(); });
    let fuse_options = matches
        .values_of("FUSE_OPTION")
        .map(|values| {
            values.fold(vec![], |mut acc, o| {
                acc.push(ffi::OsString::from("-o"));
                acc.push(ffi::OsString::from(o));
                acc
            })
        })
        .unwrap_or(vec![]);
    let cache = files::Cache::new(
        sync::RwLock::new(files::cache::Cache::new("All".into())),
    );
    let mediafs = files::MediaFS::new(cache);

    if let Err(e) = fuse_mt::mount(
        fuse_mt::FuseMT::new(mediafs, 1),
        &mount_point,
        fuse_options
            .iter()
            .map(|s| s.as_os_str())
            .collect::<Vec<&ffi::OsStr>>()
            .as_slice(),
    )
    {
        println!("Failed to mount media file system: {}", e);
        process::exit(1);
    }
}
