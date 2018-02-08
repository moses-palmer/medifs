mod cache;
pub use self::cache::{AddItemResult, Cache, Entry};

mod item;
pub use self::item::Item;

mod path;
pub use self::path::Path;

mod tag;
pub use self::tag::Tag;

mod time;
pub use self::time::{TIME_FORMAT, Timestamp};

#[cfg(test)]
pub mod tests {
    use super::*;

    use std::collections;
    use std::fs;
    use std::io::Write;
    use std::path;

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
}
