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

    /// Constructs the name to use for this entry.
    ///
    /// # Arguments
    /// *  `index` - An index to incorporate into the name in case of multiple
    ///    entries with the same name.
    ///
    /// # Panics
    /// This method will panic if passed as directory entry.
    pub fn name(&self, index: usize) -> path::PathBuf {
        match self {
            &Entry::Item(ref item) => data::name(item, item, index),
            _ => panic!(""),
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
    /// On success, the path of the new item is returned.
    ///
    /// This method will fail if an item named after the generated parent
    /// directory for `item` exists and is not a directory.
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

        self.add_item(directory, item)
    }

    /// Adds a sequence of items to the file system.
    ///
    /// This method will fail if an item named after the generated parent
    /// directory for any item exists and is not a directory.
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

    /// Adds a single item to the file system.
    ///
    /// On success, the path of the new item is returned.
    ///
    /// This method will fail if an item named after the generated parent
    /// directory for `item` exists and is not a directory.
    ///
    /// # Arguments
    /// *  `item` - The item to add.
    fn add_item<P: AsRef<path::Path>>(
        &mut self,
        directory: P,
        item: data::Item,
    ) -> AddItemResult {
        if let Some(&mut Entry::Directory(ref mut tree)) =
            self.assert_exists(&directory)
        {
            Ok(Self::add_with_index(&directory, tree, Entry::Item(item)))
        } else {
            Err(item)
        }
    }

    /// Adds an item under a tree by incrementing an index until a unique name
    /// is found.
    ///
    /// If the first attempt succeeds, no index is added.
    ///
    /// # Arguments
    /// *  `directory` - The path of the directory tree.
    /// *  `tree` - The directory tree to which to add the item.
    /// *  `entry` - The entry to add. This must not be a directory entry.
    ///
    /// # Panics
    /// This method will panic if passed as directory entry.
    fn add_with_index<P: AsRef<path::Path>>(
        directory: &P,
        tree: &mut Tree,
        entry: Entry,
    ) -> path::PathBuf {
        let directory: &path::Path = directory.as_ref();
        let mut index = 0;

        // Construct a suitable name
        let name = loop {
            let name = entry.name(index).as_os_str().to_os_string();
            if !tree.contains_key(&name) {
                break name;
            } else {
                index += 1;
            }
        };

        tree.insert(name.clone(), entry);
        directory.join(name)
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
