pub mod cache;
pub mod traits;

mod item;
pub use self::item::{shared_collection, shared_monitor, Item, ItemCollection,
                     ItemMonitor, SharedCollection, SharedMonitor};

mod path;
pub use self::path::{name, FileBase, FileExtension, Path};

mod tag;
pub use self::tag::Tag;

mod time;
pub use self::time::{system_time_to_timespec, timestamp, Timestamp};

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::collections;
    use std::fs;
    use std::io::Write;
    use std::path;
    use std::sync;

    /// Creates a simple item.
    ///
    /// # Arguments
    /// *  `name` - The item source name.
    /// *  `year` - The year part.
    /// *  `month` - The month part.
    /// *  `day` - The day part.
    pub fn item(name: &str, year: i32, month: i32, day: i32) -> Item {
        Item::new(
            path::Path::new(name),
            (year, month, day),
            collections::HashSet::new(),
        )
    }

    /// Creates an item with actual file data.
    ///
    /// # Arguments
    /// *  `path` - The file name.
    /// *  `data` - The actual file data.
    /// *  `year` - The year part.
    /// *  `month` - The month part.
    /// *  `day` - The day part.
    pub fn item_with_data<P: AsRef<path::Path>>(
        path: P,
        data: &[u8],
        year: i32,
        month: i32,
        day: i32,
    ) -> Item {
        let path: &path::Path = path.as_ref();
        fs::File::create(path)
            .and_then(|mut f| f.write(data))
            .unwrap();

        item(path.to_str().unwrap(), year, month, day)
    }

    /// A sharable list.
    pub type SharedList = sync::Arc<sync::RwLock<Vec<Item>>>;

    /// A simple item monitor that stores items in shared lists.
    pub struct Monitor {
        /// All items that have been hitherto added.
        pub added: SharedList,

        /// All items that have been hitherto removed.
        pub removed: SharedList,
    }

    impl Monitor {
        /// Creates a new monitor.
        ///
        /// # Arguments
        /// *  `added` - A list into which to store added items.
        /// *  `removed` - A list into which to store removed items.
        pub fn new(added: SharedList, removed: SharedList) -> Self {
            Self { added, removed }
        }

        /// Creates a shared list.
        pub fn list() -> SharedList {
            sync::Arc::new(sync::RwLock::new(Vec::new()))
        }
    }

    impl ItemMonitor for Monitor {
        fn item_added(&self, item: &Item) {
            self.added.write().unwrap().push(item.clone());
        }

        fn item_removed(&self, item: &Item) {
            self.removed.write().unwrap().push(item.clone());
        }
    }
}
