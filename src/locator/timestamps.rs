use std::collections;
use std::ffi;
use std::path;
use std::str;

use regex;

use super::{Entry, Locator};

use data;

/// A year, month and day.
type Day = (i32, i32, i32);

/// A tree locator for timestamped items.
pub struct TimestampsLocator;

impl Locator for TimestampsLocator {
    fn locate(
        &self,
        items: &data::SharedCollection,
        path: &mut Iterator<Item = path::Component>,
    ) -> Option<Entry> {
        let items = items.read().unwrap();
        Location::try_from(path).and_then(|location| match location {
            Location::Root => self.locate_root(&items),
            Location::Year(y) => self.locate_year(&items, y),
            Location::Month(y, m) => self.locate_month(&items, y, m),
            Location::Day(y, m, d) => self.locate_day(&items, y, m, d),
            Location::Item(i) => self.locate_item(&items, i),
        })
    }
}

impl data::ItemMonitor for TimestampsLocator {}

impl TimestampsLocator {
    /// Creates a new timestamp locator.
    pub fn new() -> Self {
        Self {}
    }

    /// Locates all root items.
    ///
    /// This is a listing of all known years.
    ///
    /// # Arguments
    /// *  `items` - The items through which to search.
    fn locate_root(&self, items: &data::ItemCollection) -> Option<Entry> {
        self.directory(
            items
                .items
                .iter()
                .map(|i| (i.timestamp.year(), 1i32, 1i32))
                .collect(),
            |&(v, _, _)| format!("{}", v).into(),
        )
    }

    /// Locates all months for a year.
    ///
    /// # Arguments
    /// *  `items` - The items through which to search.
    /// *  `year` - The year for which to list items.
    fn locate_year(
        &self,
        items: &data::ItemCollection,
        year: i32,
    ) -> Option<Entry> {
        self.directory(
            items
                .items
                .iter()
                .filter(|i| i.timestamp.year() == year)
                .map(|i| (i.timestamp.year(), i.timestamp.month(), 1i32))
                .collect(),
            |&(_, v, _)| format!("{:02}", v).into(),
        )
    }

    /// Locates all days for a month.
    ///
    /// # Arguments
    /// *  `items` - The items through which to search.
    /// *  `year` - The year for which to list items.
    /// *  `month` - The month for which to list items.
    fn locate_month(
        &self,
        items: &data::ItemCollection,
        year: i32,
        month: i32,
    ) -> Option<Entry> {
        self.directory(
            items
                .items
                .iter()
                .filter(|i| i.timestamp.year() == year)
                .filter(|i| i.timestamp.month() == month)
                .map(|i| {
                    (i.timestamp.year(), i.timestamp.month(), i.timestamp.day())
                })
                .collect(),
            |&(_, _, v)| format!("{:02}", v).into(),
        )
    }

    /// Locates all items for a day.
    ///
    /// # Arguments
    /// *  `items` - The items through which to search.
    /// *  `year` - The year for which to list items.
    /// *  `month` - The month for which to list items.
    /// *  `day` - The day for which to list items.
    fn locate_day(
        &self,
        items: &data::ItemCollection,
        year: i32,
        month: i32,
        day: i32,
    ) -> Option<Entry> {
        // Extract all items for the specific day and sort them on timestamp
        let mut items = items
            .items
            .iter()
            .filter(|i| i.timestamp.year() == year)
            .filter(|i| i.timestamp.month() == month)
            .filter(|i| i.timestamp.day() == day)
            .collect::<Vec<_>>();
        items.sort_by_key(|i| &i.timestamp);

        if !items.is_empty() {
            // If any items match, fold the list to correctly implement indice
            Some(Entry::Directory(
                items.last().unwrap().timestamp.as_ref().to_timespec(),
                items
                    .iter()
                    .fold(Vec::new(), |mut acc, i| {
                        let mut index = 0;
                        loop {
                            let name = data::name(*i, *i, index);
                            match acc.last() {
                                Some(n) if n == name.as_os_str() => {
                                    index += 1;
                                    continue;
                                }
                                _ => (),
                            }
                            acc.push(name.into_os_string());
                            break;
                        }

                        acc
                    })
                    .into_iter()
                    .collect(),
            ))
        } else {
            // If not items match, this directory does not exist
            None
        }
    }

    /// Locates a specific item.
    ///
    /// # Arguments
    /// *  `items` - The items through which to search.
    /// *  `item` - The item to locate.
    fn locate_item(
        &self,
        items: &data::ItemCollection,
        item: ItemLocator,
    ) -> Option<Entry> {
        items
            .items
            .iter()
            .filter(|i| i.timestamp.year() == item.timestamp.year())
            .filter(|i| i.timestamp.month() == item.timestamp.month())
            .filter(|i| i.timestamp.day() == item.timestamp.day())
            .skip(item.index)
            .map(|i| Entry::Item(i.clone()))
            .next()
    }

    /// Generates a generic directory entry from a collection of days.
    ///
    /// # Arguments
    /// *  `items` - The individual days.
    /// *  `mapper` - A function convering a day to a string.
    fn directory<F>(
        &self,
        items: collections::HashSet<Day>,
        mapper: F,
    ) -> Option<Entry>
    where
        F: FnMut(&Day) -> ffi::OsString,
    {
        if !items.is_empty() {
            Some(Entry::Directory(
                data::Timestamp::from(*items.iter().max().unwrap())
                    .as_ref()
                    .to_timespec(),
                items.iter().map(mapper).collect(),
            ))
        } else {
            None
        }
    }
}

/// A locator for a specific part of the tree.
#[derive(Debug, Eq, PartialEq)]
enum Location {
    /// The entire root.
    Root,

    /// An entire year.
    Year(i32),

    /// An entire month.
    Month(i32, i32),

    /// An entire day.
    Day(i32, i32, i32),

    /// A specific item.
    Item(ItemLocator),
}

impl Location {
    /// Creates a location from optional fragments.
    ///
    /// # Arguments:
    /// *  `year` - The year.
    /// *  `month` - The month. If this value is passed, `year` must also be
    ///    passed.
    /// *  `day` - The day. If this value is passed, `year` and `month` must
    ///    also be passed.
    /// *  `item` - The item. If this value is passed, `year`, `month` and
    ///    `day` must also be passed.
    pub fn new(
        year: Option<i32>,
        month: Option<i32>,
        day: Option<i32>,
        item: Option<ItemLocator>,
    ) -> Option<Self> {
        match (year, month, day, item) {
            (None, None, None, None) => Some(Location::Root),
            (Some(y), None, None, None) => Some(Location::Year(y)),
            (Some(y), Some(m), None, None) => Some(Location::Month(y, m)),
            (Some(y), Some(m), Some(d), None) => Some(Location::Day(y, m, d)),
            (Some(y), Some(m), Some(d), Some(i)) => {
                if y == i.timestamp.year() && m == i.timestamp.month()
                    && d == i.timestamp.day()
                {
                    Some(Location::Item(i))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Attempts to convert an iteration of path components to a path locator.
    ///
    /// # Arguments
    /// *  `source` - The components to convert.
    pub fn try_from(
        source: &mut Iterator<Item = path::Component>,
    ) -> Option<Self> {
        // Parse the path components; missing values will be None, and invalid
        // values will be Some(Err)
        let (year, month, day, item) = (
            source
                .next()
                .and_then(|p| p.as_os_str().to_str())
                .map(|p| p.parse::<i32>()),
            source.next().and_then(|p| p.as_os_str().to_str()).map(|p| {
                if p.len() == 2 {
                    p.parse::<i32>().map_err(|_| ())
                } else {
                    Err(())
                }
            }),
            source.next().and_then(|p| p.as_os_str().to_str()).map(|p| {
                if p.len() == 2 {
                    p.parse::<i32>().map_err(|_| ())
                } else {
                    Err(())
                }
            }),
            source
                .next()
                .and_then(|p| p.as_os_str().to_str())
                .map(|p| p.parse::<ItemLocator>()),
        );

        // If all locators present have been correctly parsed, return a value
        if year.as_ref().map(|v| v.is_ok()).unwrap_or(true)
            && month.as_ref().map(|v| v.is_ok()).unwrap_or(true)
            && day.as_ref().map(|v| v.is_ok()).unwrap_or(true)
            && item.as_ref().map(|v| v.is_ok()).unwrap_or(true)
        {
            Location::new(
                year.map(|v| v.unwrap()),
                month.map(|v| v.unwrap()),
                day.map(|v| v.unwrap()),
                item.map(|v| v.unwrap()),
            )
        } else {
            None
        }
    }
}

lazy_static! {
    /// The regular expression used to parse file names.
    static ref ITEM_LOCATOR_RE: regex::Regex = regex::Regex::new(concat!(
        r"([0-9]+)-([0-9]{2})-([0-9]{2}) ",
        r"([0-9]{2}):([0-9]{2}):([0-9]{2})",
        r"(?: \(([0-9]+)\))?",
        r"\.(.*)",
    )).unwrap();
}

/// A locator for a single item.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ItemLocator {
    /// The item timestamp.
    ///
    /// This has a resolution down to seconds.
    pub timestamp: data::Timestamp,

    /// The item index.
    pub index: usize,

    /// The file extension.
    pub extension: String,
}

impl str::FromStr for ItemLocator {
    type Err = ();

    /// Converts a string to an item locator.
    ///
    /// Strings must be on the format described by [`ITEM_LOCATOR_RE`].
    ///
    /// [`ITEM_LOCATOR_RE`]:
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        ITEM_LOCATOR_RE
            .captures(s)
            .map(|c| {
                Ok(ItemLocator {
                    timestamp: (
                        c.get(1)
                            .and_then(|s| s.as_str().parse::<i32>().ok())
                            .unwrap(),
                        c.get(2)
                            .and_then(|s| s.as_str().parse::<i32>().ok())
                            .unwrap(),
                        c.get(3)
                            .and_then(|s| s.as_str().parse::<i32>().ok())
                            .unwrap(),
                        c.get(4)
                            .and_then(|s| s.as_str().parse::<i32>().ok())
                            .unwrap(),
                        c.get(5)
                            .and_then(|s| s.as_str().parse::<i32>().ok())
                            .unwrap(),
                        c.get(6)
                            .and_then(|s| s.as_str().parse::<i32>().ok())
                            .unwrap(),
                    ).into(),
                    index: c.get(7)
                        .and_then(|s| s.as_str().parse::<usize>().ok())
                        .unwrap_or(0),
                    extension: c.get(8).map(|s| s.as_str().to_owned()).unwrap(),
                })
            })
            .unwrap_or(Err(()))
    }
}

#[cfg(test)]
mod test {
    use locator::tests::*;
    use super::*;

    /// Asserts that an item locator cannot be created from an invalid string.
    #[test]
    fn item_locator_parse_invalid_str() {
        assert_eq!(None, "invalid".parse::<ItemLocator>().ok());
        assert_eq!(None, "2000-01-01.jpg".parse::<ItemLocator>().ok());
        assert_eq!(
            None,
            "2000-01-01 01:01:01 (A).jpg".parse::<ItemLocator>().ok(),
        );
    }

    /// Asserts that an item locator can be created from a valid string.
    #[test]
    fn item_locator_parse_valid_str() {
        assert_eq!(
            ItemLocator {
                timestamp: (2000, 1, 2, 3, 4, 5).into(),
                index: 0,
                extension: String::from("ext"),
            },
            "2000-01-02 03:04:05.ext".parse::<ItemLocator>().unwrap(),
        );
        assert_eq!(
            ItemLocator {
                timestamp: (2000, 1, 2, 3, 4, 5).into(),
                index: 6,
                extension: String::from("ext"),
            },
            "2000-01-02 03:04:05 (6).ext"
                .parse::<ItemLocator>()
                .unwrap(),
        );
    }

    /// Asserts that an invalid location creation fails.
    #[test]
    fn location_new_invalid() {
        assert_eq!(None, Location::new(Some(1), None, Some(3), None));
    }

    /// Asserts that valid location cretaions succeed.
    #[test]
    fn location_new_valid() {
        assert_eq!(
            Location::Year(1),
            Location::new(Some(1), None, None, None).unwrap(),
        );
        assert_eq!(
            Location::Month(1, 2),
            Location::new(Some(1), Some(2), None, None).unwrap(),
        );
        assert_eq!(
            Location::Day(1, 2, 3),
            Location::new(Some(1), Some(2), Some(3), None).unwrap(),
        );
    }

    /// Asserts that a location cannot be created from an invalid string.
    #[test]
    fn location_from_str_invalid() {
        assert_eq!(
            None,
            Location::try_from(&mut path::PathBuf::from("a").components()),
        );
        assert_eq!(
            None,
            Location::try_from(&mut path::PathBuf::from("1/2").components()),
        );
        assert_eq!(
            None,
            Location::try_from(&mut path::PathBuf::from("1/02/3").components()),
        );
        assert_eq!(
            None,
            Location::try_from(&mut path::PathBuf::from(
                "1/02/03/1-02-03 04:05.jpeg"
            ).components()),
        );
        assert_eq!(
            None,
            Location::try_from(&mut path::PathBuf::from(
                "1/02/03/1-02-03 04:05:06 (a).jpeg"
            ).components()),
        );
        assert_eq!(
            None,
            Location::try_from(&mut path::PathBuf::from(
                "1/02/03/2-02-03 04:05:06 (7).jpeg"
            ).components()),
        );
    }

    /// Asserts that a location can be created from a valid string.
    #[test]
    fn location_from_str_valid() {
        assert_eq!(
            Location::Year(1),
            Location::try_from(&mut path::PathBuf::from("1").components())
                .unwrap(),
        );
        assert_eq!(
            Location::Month(1, 2),
            Location::try_from(&mut path::PathBuf::from("1/02").components())
                .unwrap(),
        );
        assert_eq!(
            Location::Day(1, 2, 3),
            Location::try_from(&mut path::PathBuf::from("1/02/03").components())
                .unwrap(),
        );
        assert_eq!(
            Location::Item(ItemLocator {
                timestamp: (1, 2, 3, 4, 5, 6).into(),
                index: 0,
                extension: String::from("jpeg"),
            }),
            Location::try_from(&mut path::PathBuf::from(
                "1/02/03/1-02-03 04:05:06.jpeg"
            ).components())
                .unwrap(),
        );
        assert_eq!(
            Location::Item(ItemLocator {
                timestamp: (1, 2, 3, 4, 5, 6).into(),
                index: 7,
                extension: String::from("jpeg"),
            }),
            Location::try_from(&mut path::PathBuf::from(
                "1/02/03/1-02-03 04:05:06 (7).jpeg"
            ).components())
                .unwrap(),
        );
    }

    /// Tests that a locator with no children yields nothing.
    #[test]
    fn locate_empty() {
        let locator = TimestampsLocator::new();
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
        let locator = TimestampsLocator::new();
        let items = no_items();
        items.write().unwrap().items.extend(vec![
            item("test1.jpg", 2000, 1, 1),
            item("test2.jpg", 2000, 1, 2),
        ]);
        assert_eq!(
            None,
            locator
                .locate(&items, &mut path::PathBuf::from("1999").components()),
        );
        assert_eq!(
            None,
            locator.locate(
                &items,
                &mut path::PathBuf::from("2000/02").components()
            ),
        );
    }

    /// Tests that a child can be found.
    #[test]
    fn locate_existing() {
        let locator = TimestampsLocator::new();
        let items = no_items();
        items.write().unwrap().items.extend(vec![
            item("test1.jpg", 2000, 1, 1),
            item("test2.jpg", 2000, 2, 1),
            item("test3.jpg", 2001, 1, 2),
        ]);
        assert_eq!(
            Some(Entry::Directory(
                data::Timestamp::from((2001, 1, 1)).as_ref().to_timespec(),
                [ffi::OsString::from("2000"), ffi::OsString::from("2001"),]
                    .iter()
                    .cloned()
                    .collect::<collections::HashSet<_>>(),
            )),
            locator.locate(&items, &mut path::PathBuf::from("").components(),),
        );
        assert_eq!(
            Some(Entry::Directory(
                data::Timestamp::from((2000, 2, 1)).as_ref().to_timespec(),
                [ffi::OsString::from("01"), ffi::OsString::from("02"),]
                    .iter()
                    .cloned()
                    .collect::<collections::HashSet<_>>(),
            )),
            locator
                .locate(&items, &mut path::PathBuf::from("2000").components(),),
        );
        assert_eq!(
            Some(Entry::Directory(
                data::Timestamp::from((2000, 1, 1)).as_ref().to_timespec(),
                [ffi::OsString::from("01"),]
                    .iter()
                    .cloned()
                    .collect::<collections::HashSet<_>>(),
            )),
            locator.locate(
                &items,
                &mut path::PathBuf::from("2000/01").components(),
            ),
        );
        assert_eq!(
            Some(Entry::Directory(
                data::Timestamp::from((2000, 1, 1)).as_ref().to_timespec(),
                [ffi::OsString::from("2000-01-01 00:00.jpeg"),]
                    .iter()
                    .cloned()
                    .collect::<collections::HashSet<_>>(),
            )),
            locator.locate(
                &items,
                &mut path::PathBuf::from("2000/01/01").components(),
            ),
        );
    }
}
