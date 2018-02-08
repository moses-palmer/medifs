use std::ffi;
use std::sync;

use data;


/// A wrapper for the simple data cache.
pub struct Cache {
    /// The file system cache.
    pub cache: sync::RwLock<data::Cache>,

    /// The root of the time stamped items.
    timestamp_root: ffi::OsString,
}


impl Cache {
    /// Creates a new file cache.
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
