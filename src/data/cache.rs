use std::collections;
use std::ffi;
use std::path;

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


/// A file tree cache.
#[derive(Clone, Debug)]
pub struct Cache(Entry);

impl Cache {
    /// Constructs a new empty tree cache.
    pub fn new() -> Self {
        Cache(Entry::Directory(Tree::new()))
    }

    /// Finds an entry by path.
    ///
    /// # Arguments
    /// *  `path` - The path of the entry. This must be an absolute path.
    pub fn lookup<P: AsRef<path::Path>>(&self, path: &P) -> Option<&Entry> {
        path.as_ref().components().fold(
            Some(&self.0),
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

    /// Adds a new item to this tree cache.
    ///
    /// This method will fail if any component along the path, except for the
    /// final one, exists but is not a directory. If the final component
    /// exists, it will be replaced.
    ///
    /// The item will be added under `base` in a sub directory generated from
    /// the timestamp thus: `"YYYY/MM/DD"`.
    ///
    /// # Arguments
    /// *  `base` - The base path. The actual path will depend on the item
    ///    timestamp.
    /// *  `item` - The item to add.
    pub fn add_item<P: AsRef<path::Path>>(
        &mut self,
        base: &P,
        item: data::Item,
    ) -> AddItemResult {
        let year = item.timestamp.tm_year + 1900;
        let month = item.timestamp.tm_mon + 1;
        let day = item.timestamp.tm_mday;
        let base: &path::Path = base.as_ref();
        let directory: path::PathBuf = [
            base,
            format!("{}", year).as_ref(),
            format!("{:02}", month).as_ref(),
            format!("{:02}", day).as_ref(),
        ].iter()
            .collect();

        if let Some(&mut data::Entry::Directory(ref mut tree)) =
            self.assert_exists(&directory)
        {
            let mut index = 0;
            loop {
                let name = item.name(index).as_os_str().to_os_string();
                if !tree.contains_key(&name) {
                    tree.insert(name.clone(), data::Entry::Item(item));
                    return Ok([directory, name.into()].iter().collect());
                } else {
                    index += 1;
                }
            }
        } else {
            Err(item)
        }
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
            Some(&mut self.0),
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


#[cfg(test)]
mod tests {
    use super::super::tests::*;
    use super::*;

    /// Tests that looking up an empty path yields the root.
    #[test]
    fn test_lookup_root() {
        let valid: path::PathBuf = ["/"].iter().collect();
        let cache = Cache::new();

        assert!(cache.lookup(&valid).is_some());
    }

    /// Tests that adding an item immediately under the root works.
    #[test]
    fn test_add_item_simple() {
        let mut cache = Cache::new();

        let item = item("test.jpg", 2000, 1, 1);
        let expected_path =
            path::PathBuf::from("/base/2000/01/01/2000-01-01 00:00.jpeg");
        assert_eq!(
            expected_path,
            cache.add_item(&"/base", item.clone()).unwrap(),
        );
        assert_eq!(
            Some(&Entry::Item(item)),
            cache.lookup(&expected_path),
        );
    }

    /// Tests that adding an item over a directory works.
    #[test]
    fn test_add_item_twice() {
        let mut cache = Cache::new();

        let item1 = item("test1.jpg", 2000, 1, 1);
        let expected_path1 =
            path::PathBuf::from("/base/2000/01/01/2000-01-01 00:00.jpeg");
        assert_eq!(
            expected_path1,
            cache.add_item(&"/base", item1.clone()).unwrap(),
        );

        let item2 = item("test2.jpg", 2000, 1, 1);
        let expected_path2 =
            path::PathBuf::from("/base/2000/01/01/2000-01-01 00:00 (1).jpeg");
        assert_eq!(
            expected_path2,
            cache.add_item(&"/base", item2.clone()).unwrap(),
        );
        assert_eq!(
            Some(&Entry::Item(item2)),
            cache.lookup(&expected_path2),
        );
    }
}
