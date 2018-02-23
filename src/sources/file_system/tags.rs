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


/// Information about a file.
enum ItemMeta {
    /// The file does not contain any metadata.
    Missing,

    /// A timestamp and a collection of tags.
    Present(time::Tm, collections::HashSet<data::Tag>),
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
    /// # Arguments
    /// *  `path` - The source path.
    /// *  `meta` - Image metadata.
    fn timestamp<P: AsRef<path::Path>>(
        _path: &P,
        _meta: &rexiv2::Metadata,
    ) -> time::Tm {
        unimplemented!();
    }

    /// Converts a path to a tag collection.
    ///
    /// # Arguments
    /// *  `path` - The source path.
    /// *  `meta` - Image metadata.
    fn tags(_meta: &rexiv2::Metadata) -> collections::HashSet<data::Tag> {
        unimplemented!();
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
