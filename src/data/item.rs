use std::collections;
use std::fmt;
use std::path;

use mime_guess;

use super::{FileBase, FileExtension, Tag, Timestamp};


/// A media item.
///
/// Items have an origin path, a timestamp, a set of tags and a media type.
#[derive(Clone, Debug)]
pub struct Item {
    /// The path of the source item.
    pub path: path::PathBuf,

    /// The timestamp of generation.
    pub timestamp: Timestamp,

    /// Tags applied to this item.
    pub tags: collections::HashSet<Tag>,

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
        tags: collections::HashSet<Tag>,
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

impl Eq for Item {}


impl fmt::Display for Item {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.file_base().fmt(formatter)
    }
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

        assert_eq!(
            mime::APPLICATION_OCTET_STREAM,
            item.media_type,
        );
    }

    /// Tests creation of JPEG item.
    #[test]
    fn new_jpeg() {
        let item = Item::new(
            path::Path::new("some file.jpg"),
            (2000, 01, 01, 12, 0, 0),
            collections::HashSet::new(),
        );

        assert_eq!(
            mime::IMAGE_JPEG,
            item.media_type,
        );
    }

    /// Tests stringification.
    #[test]
    fn to_string() {
        let item = Item::new(
            path::Path::new("some file.ext"),
            (2000, 01, 01, 12, 0, 0),
            collections::HashSet::new(),
        );

        assert_eq!(
            String::from("2000-01-01 12:00"),
            item.to_string(),
        );
    }
}
