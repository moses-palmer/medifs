use std::ffi;
use std::fs;
use std::io;
use std::io::{Read, Seek};
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::path;
use std::sync;

use fuse_mt;
use libc;

use data;

mod traits;
use self::traits::*;
mod util;

#[macro_use]
mod macros;


/// The actual FUSE implementation.
pub struct MediaFS {
    /// The file system cache.
    cache: sync::RwLock<data::Cache>,

    /// The root of the time stamped items.
    timestamp_root: ffi::OsString,
}


impl MediaFS {
    /// Creates a new file system instance.
    pub fn new(timestamp_root: ffi::OsString) -> Self {
        let cache = sync::RwLock::new(data::Cache::new());
        Self {
            cache,
            timestamp_root,
        }
    }

    /// Adds an item to the file system.
    ///
    /// This method locks the cache and then adds the item.
    ///
    /// On success, the path of the new item is returned.
    ///
    /// This method will fail with `Err(item)` if the lock cannot be taken, or
    /// if the item cannot be added.
    ///
    /// # Arguments
    /// *  `item` - The item to add.
    pub fn add(&self, item: data::Item) -> data::AddItemResult {
        if let Ok(mut cache) = self.cache.write() {
            cache.add_item(&self.timestamp_root, item)
        } else {
            Err(item)
        }
    }

    /// Adds a sequence of items to the file system.
    ///
    /// This method locks the cache and then adds the items.
    ///
    /// This method will fail with `Err(None)` if the lock cannot be taken, or
    /// with `Err(item)` for an item that cannot be added.
    ///
    /// # Arguments
    /// *  `items` - The items to add.
    pub fn add_iter<T: Iterator<Item = data::Item>>(
        &self,
        items: T,
    ) -> Result<(), Option<data::Item>> {
        if let Ok(mut cache) = self.cache.write() {
            items.fold(Ok(()), |acc, item| {
                acc.and_then(|_| {
                    cache
                        .add_item(&self.timestamp_root, item)
                        .map(|_| ())
                        .map_err(|item| Some(item))
                })
            })
        } else {
            Err(None)
        }
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
            &data::Entry::Directory(_) => Ok((0, 0)),
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
            &data::Entry::Item(ref item) => {
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
        let mediafs = MediaFS::new("timestamped".into());

        // Add all items
        let source_and_target_paths = items
            .map(|(name, data, year, month, day)| {
                let path = source_dir.path().join(name);
                (
                    path.clone(),
                    mount_point.path().join(
                        mediafs
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
