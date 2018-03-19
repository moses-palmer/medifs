use fuse_mt;
use libc;

use time;

use data;
use util;


impl<'a> From<&'a data::cache::Entry> for fuse_mt::ResultEntry {
    /// Converts a cache entry to a [`fuse_mt::ResultEntry`] result.
    ///
    /// The result will have relevant information read from the actual source
    /// file, but timestamps overridden by the entry timestamp.
    ///
    /// # Arguments
    /// *  `source` - The entry to convert.
    ///
    /// [`fuse_mt::ResultEntry`]: https://docs.rs/fuse_mt/0.4/fuse_mt/type.ResultEntry.html
    fn from(source: &'a data::cache::Entry) -> fuse_mt::ResultEntry {
        let ttl = time::Timespec::new(0x7FFFFFFF, 0);
        let timestamp = source.timestamp();
        match source {
            &data::cache::Entry::Directory(_) => {
                Ok((
                    ttl,
                    fuse_mt::FileAttr {
                        size: 0,
                        blocks: 0,
                        atime: timestamp,
                        mtime: timestamp,
                        ctime: timestamp,
                        crtime: timestamp,
                        kind: fuse_mt::FileType::Directory,
                        perm: 0o555,
                        nlink: 1,
                        uid: 0,
                        gid: 0,
                        rdev: 0,
                        flags: 0,
                    },
                ))
            }
            &data::cache::Entry::Item(ref item) => {
                item.path
                    .metadata()
                    .map(|meta| {
                        (
                            ttl,
                            fuse_mt::FileAttr {
                                size: meta.len(),
                                blocks: 0,
                                atime: timestamp,
                                mtime: timestamp,
                                ctime: timestamp,
                                crtime: timestamp,
                                kind: fuse_mt::FileType::RegularFile,
                                perm: 0o444,
                                nlink: 1,
                                uid: 0,
                                gid: 0,
                                rdev: 0,
                                flags: 0,
                            },
                        )
                    })
                    .map_err(util::map_error)
            }
            &data::cache::Entry::Link(_, _) => {
                Ok((
                    ttl,
                    fuse_mt::FileAttr {
                        size: 0,
                        blocks: 0,
                        atime: timestamp,
                        mtime: timestamp,
                        ctime: timestamp,
                        crtime: timestamp,
                        kind: fuse_mt::FileType::Symlink,
                        perm: 0o555,
                        nlink: 1,
                        uid: 0,
                        gid: 0,
                        rdev: 0,
                        flags: 0,
                    },
                ))
            }
        }
    }
}


impl<'a> From<&'a data::cache::Entry> for fuse_mt::ResultReaddir {
    /// Converts a cache entry to a directory listing.
    ///
    /// If the entry is not a directory, `ENOTDIR` will be returned, otherwise
    /// a listing of all child entries is returned.
    ///
    /// # Arguments
    /// *  `source` - The entry to convert.
    fn from(source: &'a data::cache::Entry) -> fuse_mt::ResultReaddir {
        match source {
            &data::cache::Entry::Directory(ref tree) => {
                Ok(
                    tree.iter()
                        .map(|(name, entry)| {
                            fuse_mt::DirectoryEntry {
                                name: name.to_os_string(),
                                kind: match entry {
                                    &data::cache::Entry::Directory(_) => {
                                        fuse_mt::FileType::Directory
                                    }
                                    &data::cache::Entry::Item(_) => {
                                        fuse_mt::FileType::RegularFile
                                    }
                                    &data::cache::Entry::Link(_, _) => {
                                        fuse_mt::FileType::Symlink
                                    }
                                },
                            }
                        })
                        .collect(),
                )
            }
            _ => Err(libc::ENOTDIR),
        }
    }
}
