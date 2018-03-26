use std;
use std::collections;
use std::path;
use std::sync;

use clap;
use rexiv2;
use time;

use data;
use files;

use super::*;
use sources::*;

file_system_base!(
    TagsSource,
    // A cache of EXIF tags already read.
    tags: sync::RwLock<collections::HashMap<path::PathBuf, ItemMeta>>,
);

/// The exiv2 tag designated for creation time.
const EXIT_TIMESTAMP_TAG_NAME: &str = &"Exif.Photo.DateTimeOriginal";

/// The format used for the creation time.
const EXIT_TIMESTAMP_TAG_FORMAT: &str = &"%Y:%m:%d %H:%M:%S";

/// The exiv2 tag designated for keywords.
const IPTC_KEYWORDS_TAG_NAME: &str = &"Iptc.Application2.Keywords";

/// Information about a file.
enum ItemMeta {
    /// The file does not contain any metadata.
    Missing,

    /// A timestamp and a collection of tags.
    Present(time::Tm, collections::HashSet<String>),
}

impl ItemMeta {
    /// Creates an item from this metadata combined with a path.
    ///
    /// # Argument
    /// *  `path` - The path of the source item.
    fn item(&self, path: &path::Path) -> data::Item {
        match self {
            &ItemMeta::Missing => path.into(),
            &ItemMeta::Present(timestamp, ref tags) => {
                data::Item::new(path, timestamp, tags.clone())
            }
        }
    }

    /// Converts a path to a timestamp.
    ///
    /// This function first attempts to read the EXIF tag, and then falls back
    /// on the file modification timestamp.
    ///
    /// # Arguments
    /// *  `path` - The source path.
    /// *  `meta` - Image metadata.
    fn timestamp<P: AsRef<path::Path>>(
        path: &P,
        meta: &rexiv2::Metadata,
    ) -> time::Tm {
        meta.get_tag_string(EXIT_TIMESTAMP_TAG_NAME)
            .ok()
            .and_then(|s| time::strptime(&s, EXIT_TIMESTAMP_TAG_FORMAT).ok())
            .unwrap_or_else(|| {
                time::at(data::system_time_to_timespec(data::timestamp(path)))
            })
    }

    /// Converts a path to a tag collection.
    ///
    /// This function first attempts to read to IPTC keywords, and then falls
    /// back on an empty set.
    ///
    /// # Arguments
    /// *  `path` - The source path.
    /// *  `meta` - Image metadata.
    fn tags(meta: &rexiv2::Metadata) -> collections::HashSet<String> {
        meta.get_tag_multiple_strings(IPTC_KEYWORDS_TAG_NAME)
            .map(|tags| {
                tags.iter()
                    .map(|s| s.parse().unwrap_or_else(|_| s.clone()))
                    .collect()
            })
            .unwrap_or_else(|_| collections::HashSet::new())
    }
}

impl<P: AsRef<path::Path>> From<P> for ItemMeta {
    /// Converts a path to item metadata.
    ///
    /// This implementation reads the ags from the source image, and if they are
    /// present constructs metadata from them, otherwise only file metadata is
    /// used.
    ///
    /// # Arguments
    /// *  `source` - The source path.
    fn from(source: P) -> Self {
        rexiv2::Metadata::new_from_path(source.as_ref())
            .map(|meta| {
                ItemMeta::Present(
                    Self::timestamp(&source, &meta),
                    Self::tags(&meta),
                )
            })
            .unwrap_or(ItemMeta::Missing)
    }
}

impl FileSystemItemGenerator for TagsSource {
    /// Generates an item from a path.
    ///
    /// # Arguments
    /// *  `path` - The path for which to generate an item.
    fn item(&self, path: &path::Path) -> data::Item {
        let key = path.to_path_buf();
        self.tags
            .write()
            .map(|mut tags| {
                tags.entry(key).or_insert_with(|| path.into()).item(&path)
            })
            .unwrap_or_else(|_| path.into())
    }
}

impl ConfigurableSource for TagsSource {
    const SUBCOMMAND_NAME: &'static str = "tags";

    fn options<'a>(app: clap::App<'a, 'a>) -> clap::App<'a, 'a> {
        options(app)
    }
}

impl ConstructableSource for TagsSource {
    fn construct<'a>(
        cache: files::Cache,
        args: &clap::ArgMatches<'a>,
    ) -> Result<Self, String> {
        Ok(TagsSource {
            cache,
            root: args.value_of(OPT_ROOT).map(|v| v.into()).unwrap(),
            timestamp: None,
            tags: sync::RwLock::new(collections::HashMap::new()),
        })
    }
}
