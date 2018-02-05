use clap;

use files;


/// A source of media files.
pub trait Source {
    /// Starts this source.
    ///
    /// This method should load all items from this source into the cache.
    fn start(&mut self);
}
