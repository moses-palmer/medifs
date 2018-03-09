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
    fn for_user(self, uid: u32, gid: u32) -> Self {
        Self { uid, gid, ..self }
    }
}
