use std::collections;
use std::fmt;
use std::path;

use mime;
use mime_guess;
use time;

use super::Tag;


/// The time format used for item timestamps.
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M";


/// A media item.
///
/// Items have an origin path, a timestamp, a set of tags and a media type.
#[derive(Clone, Debug)]
pub struct Item {
    /// The path of the source item.
    pub path: path::PathBuf,

    /// The timestamp of generation.
    pub timestamp: time::Tm,

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
    pub fn new<P: Into<path::PathBuf>>(
        path: P,
        timestamp: time::Tm,
        tags: collections::HashSet<Tag>,
    ) -> Item {
        let path: path::PathBuf = path.into();
        let media_type = mime_guess::guess_mime_type(&path);
        Item {
            path,
            timestamp,
            tags,
            media_type,
        }
    }

    /// Constructs the name to use for this item.
    ///
    /// The title
    ///
    /// # Arguments
    /// *  `index` - An index to incorporate into the name in case of multiple
    ///    items with the same name.
    pub fn name(&self, index: usize) -> path::PathBuf {
        // Unfortunately we cannot rely on mime::guess_mime_type to return the
        // preferred extension, so we explicitly handle JPEG and PNG
        let ext = if self.media_type == mime::IMAGE_JPEG {
            &"jpeg"
        } else if self.media_type == mime::IMAGE_PNG {
            &"png"
        } else {
            mime_guess::get_mime_extensions(&self.media_type)
                .and_then(|mts| mts.iter().next())
                .unwrap_or(&"bin")
        };

        if index > 0 {
            format!("{} ({}).{}", self, index, ext).into()
        } else {
            format!("{}.{}", self, ext).into()
        }
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
        write!(
            formatter,
            "{}",
            self.timestamp.strftime(TIME_FORMAT).unwrap()
        )
    }
}


#[cfg(test)]
mod tests {
    use std::path;

    use super::super::tests::*;
    use super::*;

    /// Tests creation of item of unknown type.
    #[test]
    fn new_unknown() {
        let item = Item::new(
            path::Path::new("some file.ext"),
            tm(2000, 01, 01, 12, 0, 0),
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
            tm(2000, 01, 01, 12, 0, 0),
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
            tm(2000, 01, 01, 12, 0, 0),
            collections::HashSet::new(),
        );

        assert_eq!(
            String::from("2000-01-01 12:00"),
            item.to_string(),
        );
    }

    /// Tests that the name is generated as expected.
    #[test]
    fn name() {
        let item = Item::new(
            path::Path::new("some file.jpg"),
            tm(2000, 01, 01, 12, 0, 0),
            collections::HashSet::new(),
        );

        assert_eq!(
            path::PathBuf::from("2000-01-01 12:00.jpeg"),
            item.name(0),
        );
        assert_eq!(
            path::PathBuf::from("2000-01-01 12:00 (1).jpeg"),
            item.name(1),
        );
    }
}
