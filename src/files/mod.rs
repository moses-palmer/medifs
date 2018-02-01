use fuse_mt;

mod traits;
use self::traits::*;
mod util;

#[macro_use]
mod macros;


/// The actual FUSE implementation.
pub struct MediaFS;


impl MediaFS {
    /// Creates a new file system instance.
    pub fn new() -> Self {
        Self {}
    }
}


impl fuse_mt::FilesystemMT for MediaFS {}
