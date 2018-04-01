use std::collections;
use std::ffi;

use time;

use super::*;

type Dispatch = collections::HashMap<ffi::OsString, Box<Locator>>;

/// A locator dispatching to other locators.
pub struct DispatchLocator {
    /// The dispatcher mapping.
    dispatch: Dispatch,
}

impl DispatchLocator {
    /// Consructs a new dispatch locator.
    pub fn new() -> Self {
        Self {
            dispatch: Dispatch::new(),
        }
    }

    /// Adds a locator at the specific location.
    ///
    /// # Arguments
    /// *  `at` - The path segment where to add the locator.
    /// *  `locator` - The locator to add.
    pub fn with<T: Locator + 'static>(
        mut self,
        at: ffi::OsString,
        locator: T,
    ) -> Self {
        self.dispatch.insert(at, Box::new(locator));
        self
    }
}

impl Locator for DispatchLocator {
    /// Locates an item by path components.
    ///
    /// If the path is empty, this method will return a listing of the
    /// registered locators, otherwise the first path component is used to
    /// lookup a locator to which to dispatch the remaining components.
    ///
    /// All non-normal parts at the beginning of the sequence will be skipped.
    ///
    /// # Arguments
    /// *  `items` - The source items.
    /// *  `path` - An iterator over the parts.
    fn locate(
        &self,
        items: &data::SharedCollection,
        path: &mut Iterator<Item = path::Component>,
    ) -> Option<Entry> {
        if let Some(at) = path.skip_while(|part| match part {
            &path::Component::Normal(_) => false,
            _ => true,
        }).next()
        {
            if let Some(locator) = self.dispatch.get(at.as_os_str()) {
                locator.locate(items, path)
            } else {
                None
            }
        } else {
            Some(Entry::Directory(
                // TODO: Implement
                time::Timespec::new(0, 0),
                self.dispatch.keys().cloned().collect(),
            ))
        }
    }
}

impl data::ItemMonitor for DispatchLocator {}

#[cfg(test)]
mod tests {
    use std::collections;

    use locator::tests::*;
    use super::*;

    /// Tests that a locator with no children yields nothing.
    #[test]
    fn locate_empty() {
        let locator = DispatchLocator::new();
        let items = no_items();
        assert_eq!(
            None,
            locator
                .locate(&items, &mut path::PathBuf::from("test").components()),
        );
    }

    /// Tests that an unknown child yields nothing.
    #[test]
    fn locate_non_existing() {
        let locator = DispatchLocator::new()
            .with("test1".into(), DummyLocator::new())
            .with("test2".into(), DummyLocator::new());
        let items = no_items();
        assert_eq!(
            None,
            locator
                .locate(&items, &mut path::PathBuf::from("test3").components()),
        );
    }

    /// Tests that the root item yields all registered children.
    #[test]
    fn locate_root() {
        let locator = DispatchLocator::new()
            .with("test1".into(), DummyLocator::new())
            .with("test2".into(), DummyLocator::new());
        let items = no_items();
        match locator.locate(&items, &mut path::PathBuf::from("/").components())
        {
            Some(Entry::Directory(_, tree)) => assert_eq!(
                vec![
                    ffi::OsString::from("test1"),
                    ffi::OsString::from("test2"),
                ].into_iter()
                    .collect::<collections::HashSet<_>>(),
                tree,
            ),
            e => panic!(format!("{:?} was unexpected", e)),
        }
    }

    /// Tests a shallow locate.
    #[test]
    fn locate_shallow() {
        let locator =
            DispatchLocator::new().with("test".into(), DummyLocator::new());
        let items = no_items();
        match locator
            .locate(&items, &mut path::PathBuf::from("/test").components())
        {
            Some(Entry::Directory(_, tree)) => assert_eq!(
                vec![ffi::OsString::from("0")]
                    .into_iter()
                    .collect::<collections::HashSet<_>>(),
                tree,
            ),
            e => panic!(format!("{:?} was unexpected", e)),
        }
    }

    /// Tests a deep locate.
    #[test]
    fn locate_deep() {
        let locator =
            DispatchLocator::new().with("test".into(), DummyLocator::new());
        let items = no_items();
        match locator
            .locate(&items, &mut path::PathBuf::from("/test/0/1").components())
        {
            Some(Entry::Directory(_, tree)) => assert_eq!(
                vec![ffi::OsString::from("2")]
                    .into_iter()
                    .collect::<collections::HashSet<_>>(),
                tree,
            ),
            e => panic!(format!("{:?} was unexpected", e)),
        }
    }

    /// Tests a deep locate with missing value.
    #[test]
    fn locate_deep_missing() {
        let locator =
            DispatchLocator::new().with("test".into(), DummyLocator::new());
        let items = no_items();
        assert_eq!(
            None,
            locator.locate(
                &items,
                &mut path::PathBuf::from("/test/0/2").components()
            ),
        );
    }
}
