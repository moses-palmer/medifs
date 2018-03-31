use std::collections;
use std::ffi;
use std::path;

use time;

use data;

mod dispatch;
pub use self::dispatch::DispatchLocator;

mod timestamps;
pub use self::timestamps::TimestampsLocator;

/// A directory tree.
pub type Tree = collections::HashSet<ffi::OsString>;

/// A cache entry.
#[derive(Clone, Debug, PartialEq)]
pub enum Entry {
    /// A directory entry.
    Directory(time::Timespec, Tree),

    /// A link to another item.
    Link(time::Timespec, ffi::OsString),

    /// An item entry. The value is the item.
    Item(data::Item),
}

impl Entry {
    /// Returns the latest timestamp selected from this and child entries.
    pub fn timestamp(&self) -> time::Timespec {
        match self {
            &Entry::Directory(timestamp, _) | &Entry::Link(timestamp, _) => {
                timestamp
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
    /// This method will panic if passed a directory entry.
    pub fn name(&self, index: usize) -> path::PathBuf {
        match self {
            &Entry::Item(ref item) => data::name(item, item, index),
            &Entry::Link(_, ref path) => data::name(path, path, index),
            _ => panic!(""),
        }
    }
}

/// A proxy for some tree structure.
pub trait Locator {
    /// Locates an item by path components.
    ///
    /// # Arguments
    /// *  `items` - The source items.
    /// *  `path` - An iterator over the parts.
    fn locate(
        &self,
        items: &data::SharedCollection,
        path: &mut Iterator<Item = path::Component>,
    ) -> Option<Entry>;
}

#[cfg(test)]
mod tests {
    pub use data::tests::*;
    use data;

    pub use super::*;

    /// Creates an empty shared collection with a dummy monitor.
    pub fn no_items() -> data::SharedCollection {
        data::shared_collection(data::shared_monitor(Monitor::new(
            Monitor::list(),
            Monitor::list(),
        )))
    }

    /// A dummy locator.
    ///
    /// A dummy locator resolves paths on the form `"0/1/2/..."`.
    pub struct DummyLocator;

    impl DummyLocator {
        /// Creates a new dummy locator.
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Locator for DummyLocator {
        fn locate<'a>(
            &self,
            _items: &data::SharedCollection,
            path: &mut Iterator<Item = path::Component>,
        ) -> Option<Entry> {
            path.enumerate()
                .fold(Some(0), |acc, (i, part)| {
                    acc.and_then(|_| {
                        part.as_os_str()
                            .to_str()
                            .unwrap()
                            .parse::<usize>()
                            .ok()
                            .and_then(
                                |p| if p == i { Some(i + 1) } else { None },
                            )
                    })
                })
                .map(|i| {
                    Entry::Directory(
                        time::Timespec::new(0, 0),
                        vec![ffi::OsString::from(i.to_string())]
                            .into_iter()
                            .collect(),
                    )
                })
        }
    }
}
