use std::fs;
use std::io;
use std::io::{Read, Seek};
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::path;
use std::sync;

use fuse_mt;
use libc;

pub mod cache;

mod traits;
use self::traits::*;
mod util;

#[macro_use]
mod macros;

/// The type used as cache.
pub type Cache = sync::Arc<sync::RwLock<cache::Cache>>;


/// The actual FUSE implementation.
pub struct MediaFS {
    /// The backing file system cache.
    cache: Cache,
}

impl MediaFS {
    /// Creates a new file system instance.
    pub fn new(cache: Cache) -> MediaFS {
        Self { cache }
    }
}


impl fuse_mt::FilesystemMT for MediaFS {
    fn init(&self, _req: fuse_mt::RequestInfo) -> fuse_mt::ResultEmpty {
        Ok(())
    }

    fn getattr(
        &self,
        req: fuse_mt::RequestInfo,
        path: &path::Path,
        _fh: Option<u64>,
    ) -> fuse_mt::ResultEntry {
        let result: fuse_mt::ResultEntry = lookup!(cache!(self.cache), &path)
            .into();
        result.map(|(ttl, fa)| (ttl, fa.for_user(req.uid, req.gid)))
    }

    fn opendir(
        &self,
        _req: fuse_mt::RequestInfo,
        path: &path::Path,
        _flags: u32,
    ) -> fuse_mt::ResultOpen {
        match lookup!(cache!(self.cache), &path) {
            &cache::Entry::Directory(_) => Ok((0, 0)),
            _ => Err(libc::ENOTDIR),
        }
    }

    fn readdir(
        &self,
        _req: fuse_mt::RequestInfo,
        path: &path::Path,
        _fh: u64,
    ) -> fuse_mt::ResultReaddir {
        lookup!(cache!(self.cache), &path).into()
    }

    fn open(
        &self,
        _req: fuse_mt::RequestInfo,
        path: &path::Path,
        flags: u32,
    ) -> fuse_mt::ResultOpen {
        match lookup!(cache!(self.cache), &path) {
            &cache::Entry::Item(ref item) => {
                fs::File::open(&item.path)
                    .map(|f| (f.into_raw_fd() as u64, flags))
                    .map_err(util::map_error)
            }
            _ => Err(libc::EINVAL),
        }
    }

    fn read(
        &self,
        _req: fuse_mt::RequestInfo,
        _path: &path::Path,
        fh: u64,
        offset: u64,
        size: u32,
    ) -> fuse_mt::ResultData {
        // Recreate file
        let mut file = unsafe { fs::File::from_raw_fd(fh as i32) };

        // Read the file
        let result = file.seek(io::SeekFrom::Start(offset))
            .and_then(|_| {
                let mut buffer = vec![0u8; size as usize];
                file.read(&mut buffer).map(|size| {
                    buffer.resize(size, 0u8);
                    buffer
                })
            })
            .map_err(util::map_error);

        // Release file
        file.into_raw_fd();

        result
    }

    fn release(
        &self,
        _req: fuse_mt::RequestInfo,
        _path: &path::Path,
        fh: u64,
        _flags: u32,
        _lock_owner: u64,
        _flush: bool,
    ) -> fuse_mt::ResultEmpty {
        // Recreate file and drop it
        unsafe { fs::File::from_raw_fd(fh as i32) };

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;
    use std::sync;

    use fuse;
    use tempdir;

    use super::*;
    use data::tests::*;


    /// Tests that getattr returns the expected data.
    #[test]
    fn test_getattr() {
        let data = "hello world";
        let (mount_point, _source_dir, _session, paths) =
            mount(vec![("test.jpg", data, 2000, 1, 1)].into_iter());

        let (ref source, ref target) = paths[0];
        let source_meta = source.metadata().unwrap();
        let target_meta = target.metadata().unwrap();
        assert_eq!(
            io::Error::from_raw_os_error(libc::ENOENT).kind(),
            mount_point.path().join(
                "invalid/path"
            ).metadata().unwrap_err().kind(),
        );
        assert_eq!(
            0o444,
            target_meta.permissions().mode() & 0o777,
        );
        assert_eq!(
            source_meta.len(),
            target_meta.len(),
        );
    }

    /// Tests that reading from the FUSE file system yields the same data as
    /// reading from the actual file.
    #[test]
    fn test_readdir() {
        let data = "hello world";
        let (mount_point, _source_dir, _session, paths) =
            mount(
                vec![
                    ("test1.jpg", data, 2000, 1, 1),
                    ("test2.jpg", data, 2000, 1, 1),
                    ("test3.jpg", data, 2000, 1, 2),
                ].into_iter(),
            );

        let (_, ref directory1) = paths[0];
        assert_eq!(
            io::Error::from_raw_os_error(libc::ENOENT).kind(),
            fs::read_dir(
                mount_point.path().join("invalid/path")
            ).unwrap_err().kind(),
        );
        assert_eq!(
            io::Error::from_raw_os_error(libc::ENOTDIR).kind(),
            fs::read_dir(directory1).unwrap_err().kind(),
        );
        assert_eq!(
            2,
            fs::read_dir(directory1.parent().unwrap()).unwrap().count(),
        );

        let (_, ref directory2) = paths[2];
        assert_eq!(
            1,
            fs::read_dir(directory2.parent().unwrap()).unwrap().count(),
        );
    }

    /// Tests that reading from the FUSE file system yields the same data as
    /// reading from the actual file.
    #[test]
    fn test_read() {
        let data = "hello world";
        let (mount_point, _source_dir, _session, paths) =
            mount(vec![("test.jpg", data, 2000, 1, 1)].into_iter());

        let (ref source, ref target) = paths[0];
        assert_eq!(
            io::Error::from_raw_os_error(libc::ENOENT).kind(),
            fs::File::open(
                mount_point.path().join("invalid/path"),
            ).unwrap_err().kind(),
        );
        assert_eq!(
            read_file(source),
            read_file(target),
        );
    }

    /// An item to populate a file system.
    ///
    /// This is the tuple `(name, data, year, moth, day)`.
    type MountItem = (&'static str, &'static str, i32, i32, i32);

    /// The result of a mount operation.
    ///
    /// This is the tuple `(mount_point, source_dir, background_session,
    /// source_and_target_paths)`
    type MountResult<'a> = (tempdir::TempDir,
                            tempdir::TempDir,
                            fuse::BackgroundSession<'a>,
                            Vec<(path::PathBuf, path::PathBuf)>);

    /// Mounts a file system on a temporary mount point.
    ///
    /// The mount point and a temporary directory is returned along with a
    /// background session.
    ///
    /// # Arguments
    /// *  `items` - A sequence of
    ///    used to populate the file system.
    fn mount<'a, T: Iterator<Item = MountItem>>(items: T) -> MountResult<'a> {
        // Create temporary directories and the file system handler
        let mount_point = tempdir::TempDir::new(&"medifs-mount").unwrap();
        let source_dir = tempdir::TempDir::new(&"medifs-source").unwrap();
        let cache =
            Cache::new(sync::RwLock::new(cache::Cache::new("All".into())));
        let mediafs = MediaFS::new(cache.clone());

        // Add all items
        let source_and_target_paths = items
            .map(|(name, data, year, month, day)| {
                let path = source_dir.path().join(name);
                (
                    path.clone(),
                    mount_point.path().join(
                        cache
                            .write()
                            .unwrap()
                            .add(item_with_data(
                                path,
                                data.as_bytes(),
                                year,
                                month,
                                day,
                            ))
                            .unwrap(),
                    ),
                )
            })
            .collect::<Vec<_>>();

        // Actually mount the file system
        let background_session = unsafe {
            fuse_mt::spawn_mount(
                fuse_mt::FuseMT::new(mediafs, 1),
                &mount_point,
                &[],
            ).unwrap()
        };

        (
            mount_point,
            source_dir,
            background_session,
            source_and_target_paths,
        )
    }

    /// Reads the full content of a file into a buffer.
    ///
    /// # Arguments
    /// *  `path` - The file path.
    fn read_file<P: AsRef<path::Path>>(path: P) -> Vec<u8> {
        let mut file = fs::File::open(&path).unwrap();
        let mut result = vec![0u8; file.metadata().unwrap().len() as usize];
        file.read_to_end(&mut result).unwrap();

        result
    }
}
