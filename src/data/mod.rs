mod cache;
pub use self::cache::{AddItemResult, Cache, Entry};

mod item;
pub use self::item::Item;

mod path;
pub use self::path::Path;

mod tag;
pub use self::tag::Tag;

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::collections;
    use std::fs;
    use std::io::Write;
    use std::path;

    use time;

    /// Creates a simple time structure.
    ///
    /// # Arguments
    /// *  `year` - The year part.
    /// *  `month` - The month part.
    /// *  `day` - The day part.
    /// *  `hour` - The hour part.
    /// *  `min` - The minute part.
    /// *  `sec` - The second part.
    pub fn tm(
        year: i32,
        month: i32,
        day: i32,
        hour: i32,
        min: i32,
        sec: i32,
    ) -> time::Tm {
        let tm_year = year - 1900;
        let tm_mon = month - 1;
        let tm_mday = day;
        let tm_hour = hour;
        let tm_min = min;
        let tm_sec = sec;
        time::Tm {
            tm_year,
            tm_mon,
            tm_mday,
            tm_hour,
            tm_min,
            tm_sec,
            tm_nsec: 0,
            tm_yday: 0,
            tm_wday: 0,
            tm_isdst: -1,
            tm_utcoff: 0,
        }
    }

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
            tm(year, month, day, 0, 0, 0),
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
}
