use std::collections;
use std::ffi;
use std::path;

use time;

use data;


/// A directory tree.
pub type Tree = collections::HashMap<ffi::OsString, Entry>;


/// The result of an item addition.
pub type AddItemResult = Result<path::PathBuf, data::Item>;


/// A cache entry.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    /// A directory entry. The value is the child entries.
    Directory(Tree),

    /// An item entry. The value is the item.
    Item(data::Item),
}

impl Entry {
    /// Returns the latest timestamp selected from this and child entries.
    pub fn timestamp(&self) -> time::Timespec {
        match self {
            &Entry::Directory(ref tree) => {
                tree.values()
                    .max_by(|a, b| a.timestamp().cmp(&b.timestamp()))
                    .map(|entry| entry.timestamp())
                    .unwrap_or(time::Timespec::new(0, 0))
            }
            &Entry::Item(ref item) => item.timestamp.as_ref().to_timespec(),
        }
    }

    /// Clears this entry and its child entries.
    ///
    /// This has an effect only on directory entries.
    pub fn clear(&mut self) {
        match self {
            &mut Entry::Directory(ref mut tree) => tree.clear(),
            _ => (),
        }
    }
}


/// A wrapper for the simple data cache.
pub struct Cache {
    /// The root directory.
    root: Entry,

    /// The root of the time stamped items.
    timestamp_root: ffi::OsString,
}


impl Cache {
    /// Creates a new file cache.
    pub fn new(timestamp_root: ffi::OsString) -> Self {
        let root = Entry::Directory(Tree::new());
        Self {
            root,
            timestamp_root,
        }
    }

    /// Finds an entry by path.
    ///
    /// # Arguments
    /// *  `path` - The path of the entry. This must be an absolute path.
    pub fn lookup<P: AsRef<path::Path>>(&self, path: &P) -> Option<&Entry> {
        path.as_ref().components().fold(
            Some(&self.root),
            |acc, part| match part {
                // The root will be the first component, so we return the
                // initial value unchanged, and for the current directory we do
                // the same
                path::Component::RootDir | path::Component::CurDir => acc,

                // Go deeper for normal files
                path::Component::Normal(path) => {
                    acc.and_then(|entry| match entry {
                        &Entry::Directory(ref tree) => {
                            tree.get(path.into()).clone()
                        }
                        _ => None,
                    })
                }

                // We support only absolute paths without special directories
                _ => None,
            },
        )
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
    pub fn add(&mut self, item: data::Item) -> AddItemResult {
        let directory: path::PathBuf = {
            let base: &path::Path = self.timestamp_root.as_ref();
            [
                base,
                format!("{}", item.timestamp.year()).as_ref(),
                format!("{:02}", item.timestamp.month()).as_ref(),
                format!("{:02}", item.timestamp.day()).as_ref(),
            ].iter()
                .collect()
        };

        if let Some(&mut Entry::Directory(ref mut tree)) =
            self.assert_exists(&directory)
        {
            let mut index = 0;
            loop {
                let name = item.name(index).as_os_str().to_os_string();
                if !tree.contains_key(&name) {
                    tree.insert(name.clone(), Entry::Item(item));
                    return Ok([directory, name.into()].iter().collect());
                } else {
                    index += 1;
                }
            }
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
        &mut self,
        items: T,
    ) -> Result<(), Option<data::Item>> {
        items.fold(Ok(()), |acc, item| {
            acc.and_then(
                |_| self.add(item).map(|_| ()).map_err(|item| Some(item)),
            )
        })
    }

    /// Replaces all items in the file system.
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
        self.root.clear();
        items.fold(Ok(()), |acc, item| {
            acc.and_then(
                |_| self.add(item).map(|_| ()).map_err(|item| Some(item)),
            )
        })
    }

    /// Asserts that a path exists.
    ///
    /// All missing parents will be created. If a non-directory entry is
    /// encountered along the way, except for the final component, `None` is
    /// returned.
    ///
    /// If the final part has to be created, it will be created as a directory.
    ///
    /// # Arguments
    /// *  `path` - The full path.
    fn assert_exists<P: AsRef<path::Path>>(
        &mut self,
        path: &P,
    ) -> Option<&mut Entry> {
        path.as_ref().components().fold(
            Some(&mut self.root),
            |acc, part| match part {
                // The root will be the first component, so we return the
                // initial value unchanged, and for the current directory we do
                // the same
                path::Component::RootDir | path::Component::CurDir => acc,

                // Go deeper for normal files
                path::Component::Normal(path) => {
                    acc.and_then(|entry| match *entry {
                        Entry::Directory(ref mut tree) => {
                            Some(
                                tree.entry(path.to_os_string()).or_insert_with(
                                    || {
                                        Entry::Directory(Tree::new())
                                    },
                                ),
                            )
                        }
                        _ => None,
                    })
                }

                // We support only absolute paths without special directories
                _ => None,
            },
        )
    }
}
