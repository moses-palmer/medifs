use fuse_mt;

/// Allows changing the owner of a resource.
pub trait ForUser {
    /// Changes the owner and group of this item.
    ///
    /// # Arguments
    /// *  `uid` - The user ID.
    /// *  `gid` - The group ID.
    fn for_user(self, uid: u32, gid: u32) -> Self;
}

impl ForUser for fuse_mt::FileAttr {
    /// Changes the permissions of a `FileAttr` by replacing the [`uid`] and
    /// [`gid`] fields.
    ///
    /// # Arguments
    /// *  `uid` - the user id.
    /// *  `gid` - the group id.
    ///
    /// [`uid`]: https://docs.rs/fuse_mt/0.4/fuse_mt/struct.FileAttr.html#structfield.uid
    /// [`gid`]: https://docs.rs/fuse_mt/0.4/fuse_mt/struct.FileAttr.html#structfield.gid
    fn for_user(self, uid: u32, gid: u32) -> Self {
        Self { uid, gid, ..self }
    }
}
