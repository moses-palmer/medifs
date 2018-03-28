use std::cmp;
use std::collections;
use std::fmt;
use std::path;
use std::sync;

use mime_guess;

use super::{FileBase, FileExtension, Timestamp};

/// A media item.
///
/// Items have an origin path, a timestamp, a set of tags and a media type.
#[derive(Clone, Debug, Eq)]
pub struct Item {
    /// The path of the source item.
    pub path: path::PathBuf,

    /// The timestamp of generation.
    pub timestamp: Timestamp,

    /// Tags applied to this item.
    pub tags: collections::HashSet<String>,

    /// The media type, guessed from the file extension.
    pub media_type: mime_guess::Mime,
}

impl Item {
    /// Creates a new item.
    ///
    /// This method guesses the media type based on the file name.
    ///
    /// # Arguments
    /// *  `path` - The path of the source item.
    /// *  `timestamp` - The time stamp of generation.
    /// *  `tags` - Tag applied to this item.
    pub fn new<P: Into<path::PathBuf>, T: Into<Timestamp>>(
        path: P,
        timestamp: T,
        tags: collections::HashSet<String>,
    ) -> Self {
        let path: path::PathBuf = path.into();
        let timestamp: Timestamp = timestamp.into();
        let media_type = mime_guess::guess_mime_type(&path);
        Item {
            path,
            timestamp,
            tags,
            media_type,
        }
    }
}

impl FileBase for Item {
    type T = Timestamp;

    fn file_base(&self) -> Self::T {
        self.timestamp.clone()
    }
}

impl FileExtension for Item {
    type T = &'static str;

    fn file_extension(&self) -> Self::T {
        self.media_type.file_extension()
    }
}

impl PartialEq for Item {
    fn eq(&self, other: &Item) -> bool {
        self.path.eq(&other.path)
    }
}

impl fmt::Display for Item {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.file_base().fmt(formatter)
    }
}

/// A monitor for item collections.
pub trait ItemMonitor: Send + Sync {
    /// An item has been added.
    ///
    /// # Arguments
    /// *  `_item` - The item.
    fn item_added(&self, _item: &Item) {}

    /// An item has been removed.
    ///
    /// # Arguments
    /// *  `_item` - The item.
    fn item_removed(&self, _item: &Item) {}
}

/// A sharable item monitor.
pub type SharedMonitor = sync::Arc<sync::RwLock<Box<ItemMonitor>>>;

impl ItemMonitor for SharedMonitor {
    fn item_added(&self, item: &Item) {
        self.read().unwrap().item_added(item);
    }

    fn item_removed(&self, item: &Item) {
        self.read().unwrap().item_removed(item);
    }
}

/// Constructs a sharable item monitor from an owned monitor.
///
/// # Arguments
/// *  `monitor` - The monitor
pub fn shared_monitor<T: ItemMonitor + 'static>(monitor: T) -> SharedMonitor {
    sync::Arc::new(sync::RwLock::new(Box::new(monitor)))
}

/// A collection of items.
///
/// Apart from holding a collection of items, this type allows registering a
/// monitor that is notified when the collection changes.
pub struct ItemCollection {
    /// The items.
    pub items: Vec<Item>,

    /// The registered monitor.
    monitor: SharedMonitor,
}

impl ItemCollection {
    /// Creates a new item collection.
    ///
    /// # Arguments
    /// *  `monitor` - The item monitor.
    pub fn new(monitor: SharedMonitor) -> Self {
        Self {
            items: vec![],
            monitor,
        }
    }

    /// Adds an item to this item collection.
    ///
    /// Calling this method will cause [`ItemMonitor::item_added`] to be called
    /// for the registered monitor.
    ///
    /// When the callback is called, the item has not yet been added.
    ///
    /// # Arguments
    /// *  `item` - The item to add.
    ///
    /// [`ItemMonitor::item_added`]: trait.ItemMonitor.html#method.item_added
    pub fn add(&mut self, item: Item) {
        self.monitor.item_added(&item);
        self.items.push(item);
    }

    /// Removes an item from this item collection.
    ///
    /// Calling this method will cause [`ItemMonitor::item_removed`] to be
    /// called if a matching item is found.
    ///
    /// Only the first item is removed.
    ///
    /// # Arguments
    /// *  `path` - The source path of the item to remove.
    ///
    /// [`ItemMonitor::item_removed`]: trait.ItemMonitor.html#method.item_removed
    pub fn remove_by_path<P>(&mut self, path: &P) -> Option<Item>
    where
        P: cmp::PartialEq<path::PathBuf>,
    {
        if let Some((index, _)) = self.items
            .iter()
            .enumerate()
            .filter(|&(_, item)| path == &item.path)
            .next()
        {
            let item = self.items.remove(index);
            self.monitor.item_removed(&item);
            Some(item)
        } else {
            None
        }
    }
}

/// A sharable item collection.
pub type SharedCollection = sync::Arc<sync::RwLock<ItemCollection>>;

/// Constructs a sharable item monitor from an owned monitor.
///
/// # Arguments
/// *  `monitor` - The monitor
pub fn shared_collection(monitor: SharedMonitor) -> SharedCollection {
    sync::Arc::new(sync::RwLock::new(ItemCollection::new(monitor)))
}

#[cfg(test)]
mod tests {
    use std::path;

    use mime;

    use super::*;

    /// Tests creation of item of unknown type.
    #[test]
    fn new_unknown() {
        let item = Item::new(
            path::Path::new("some file.ext"),
            (2000, 01, 01, 12, 0, 0),
            collections::HashSet::new(),
        );

        assert_eq!(mime::APPLICATION_OCTET_STREAM, item.media_type);
    }

    /// Tests creation of JPEG item.
    #[test]
    fn new_jpeg() {
        let item = Item::new(
            path::Path::new("some file.jpg"),
            (2000, 01, 01, 12, 0, 0),
            collections::HashSet::new(),
        );

        assert_eq!(mime::IMAGE_JPEG, item.media_type);
    }

    /// Tests stringification.
    #[test]
    fn to_string() {
        let item = Item::new(
            path::Path::new("some file.ext"),
            (2000, 01, 01, 12, 0, 0),
            collections::HashSet::new(),
        );

        assert_eq!(String::from("2000-01-01 12:00"), item.to_string());
    }

    /// Tests adding an item.
    #[test]
    fn add_item() {
        let added = Monitor::list();
        let removed = Monitor::list();
        let mut collection = ItemCollection::new(shared_monitor(
            Monitor::new(added.clone(), removed.clone()),
        ));
        let items = vec![
            Item::new(
                path::Path::new("some file.ext"),
                (2000, 01, 01, 12, 0, 0),
                collections::HashSet::new(),
            ),
            Item::new(
                path::Path::new("some other file.ext"),
                (2000, 01, 01, 12, 1, 0),
                collections::HashSet::new(),
            ),
        ];

        items.iter().for_each(|i| collection.add(i.clone()));
        assert_eq!(items, collection.items);
        assert_eq!(items, *added.read().unwrap());
    }

    /// Tests removing an item.
    #[test]
    fn remove_item() {
        let added = Monitor::list();
        let removed = Monitor::list();
        let mut collection = ItemCollection::new(shared_monitor(
            Monitor::new(added.clone(), removed.clone()),
        ));
        let items = vec![
            Item::new(
                path::Path::new("some file.ext"),
                (2000, 01, 01, 12, 0, 0),
                collections::HashSet::new(),
            ),
            Item::new(
                path::Path::new("some other file.ext"),
                (2000, 01, 01, 12, 1, 0),
                collections::HashSet::new(),
            ),
        ];

        items.iter().for_each(|i| collection.add(i.clone()));
        assert_eq!(
            Some(items[0].clone()),
            collection.remove_by_path(&items[0].path),
        );
        assert_eq!(vec![items[1].clone()], collection.items);
        assert_eq!(items, *added.read().unwrap());
        assert_eq!(vec![items[0].clone()], *removed.read().unwrap());
    }

    type SharedList = sync::Arc<sync::RwLock<Vec<Item>>>;

    struct Monitor {
        pub added: SharedList,
        pub removed: SharedList,
    }

    impl Monitor {
        pub fn new(added: SharedList, removed: SharedList) -> Self {
            Self { added, removed }
        }

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
