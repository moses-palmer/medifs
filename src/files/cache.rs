use std::ffi;
use std::path;

use data;


/// A wrapper for the simple data cache.
pub struct Cache {
    /// The file system cache.
    pub cache: data::Cache,

    /// The root of the time stamped items.
    timestamp_root: ffi::OsString,
}


impl Cache {
    /// Creates a new file cache.
    pub fn new(timestamp_root: ffi::OsString) -> Self {
        let cache = data::Cache::new();
        Self {
            cache,
            timestamp_root,
        }
    }

    /// Finds an entry by path.
    ///
    /// # Arguments
    /// *  `path` - The path of the entry. This must be an absolute path.
    pub fn lookup<P: AsRef<path::Path>>(
        &self,
        path: &P,
    ) -> Option<&data::Entry> {
        self.cache.lookup(path)
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
    pub fn add(&mut self, item: data::Item) -> data::AddItemResult {
        self.cache.add_item(&self.timestamp_root, item)
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
        &mut self,
        items: T,
    ) -> Result<(), Option<data::Item>> {
        items.fold(Ok(()), |acc, item| {
            acc.and_then(|_| {
                self.cache
                    .add_item(&self.timestamp_root, item)
                    .map(|_| ())
                    .map_err(|item| Some(item))
            })
        })
    }

    /// Replaces all items in the file system.
    ///
    /// This method locks the cachei, clears it and then adds the items.
    ///
    /// This method will fail with `Err(None)` if the lock cannot be taken, or
    /// with `Err(item)` for an item that cannot be added.
    ///
    /// # Arguments
    /// *  `items` - The items to add.
    pub fn replace_all<T: Iterator<Item = data::Item>>(
        &mut self,
        items: T,
    ) -> Result<(), Option<data::Item>> {
        self.cache.clear();
        items.fold(Ok(()), |acc, item| {
            acc.and_then(|_| {
                self.cache
                    .add_item(&self.timestamp_root, item)
                    .map(|_| ())
                    .map_err(|item| Some(item))
            })
        })
    }
}
