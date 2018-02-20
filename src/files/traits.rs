use fuse_mt;
use libc;

use time;

use files;
use super::util;


/// Allows changing the owner of a resource.
pub trait ForUser {
    /// Changes the owner and group of this item.
    ///
    /// # Arguments
    /// *  `uid` - The user ID.
    /// *  `gid` - The group ID.
    fn for_user(self, uid: u32, gid: u32) -> Self;
}


impl<'a> From<&'a files::cache::Entry> for fuse_mt::ResultEntry {
    fn from(source: &'a files::cache::Entry) -> fuse_mt::ResultEntry {
        let ttl = time::Timespec::new(0x7FFFFFFF, 0);
        let timestamp = source.timestamp();
        match source {
            &files::cache::Entry::Directory(_) => {
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
            &files::cache::Entry::Item(ref item) => {
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
        }

    }
}

impl ForUser for fuse_mt::FileAttr {
    fn for_user(self, uid: u32, gid: u32) -> Self {
        Self { uid, gid, ..self }
    }
}


impl<'a> From<&'a files::cache::Entry> for fuse_mt::ResultReaddir {
    fn from(source: &'a files::cache::Entry) -> fuse_mt::ResultReaddir {
        match source {
            &files::cache::Entry::Directory(ref tree) => {
                Ok(
                    tree.iter()
                        .map(|(name, entry)| {
                            fuse_mt::DirectoryEntry {
                                name: name.to_os_string(),
                                kind: match entry {
                                    &files::cache::Entry::Directory(_) => {
                                        fuse_mt::FileType::Directory
                                    }
                                    &files::cache::Entry::Item(_) => {
                                        fuse_mt::FileType::RegularFile
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
