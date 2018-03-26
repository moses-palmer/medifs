use std;
use std::collections;
use std::path;

use clap;
use mime_guess;
use walkdir;

use data;
use files;

#[macro_use]
mod macros;

mod directory;
pub use self::directory::*;

mod tags;
pub use self::tags::*;

/// The name of the argument specifying the root.
const OPT_ROOT: &'static str = &"ROOT";

/// Adds the base options for a file system source.
///
/// # Arguments
/// *  `app` - The application to which to add the arguments.
fn options<'a>(app: clap::App<'a, 'a>) -> clap::App<'a, 'a> {
    app.arg(
        clap::Arg::with_name(OPT_ROOT)
            .help("The source directory.")
            .required(true),
    )
}

/// Generates an item from a path.
///
/// This trait must be implemented by file system sources.
pub trait FileSystemItemGenerator {
    /// Generates an item from a path.
    ///
    /// # Arguments
    /// *  `path` - The path for which to generate an item.
    fn item(&self, path: &path::Path) -> data::Item;
}

pub trait FileSystemSource: super::Source + FileSystemItemGenerator {
    /// Populates the cache with items from this source.
    ///
    /// This method will completely replace the items.
    ///
    /// # Panics
    /// If an item fails to be added.
    fn populate(&self) {
        if let Ok(mut cache) = self.cache().write() {
            // Ignore errors when listing and ignore non-image files
            cache
                .replace_all(
                    walkdir::WalkDir::new(self.root())
                        .into_iter()
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            mime_guess::guess_mime_type(e.path()).type_()
                                == "image"
                        })
                        .map(|e| self.item(e.path())),
                )
                .unwrap();
        }
    }

    /// The cache.
    fn cache(&self) -> &files::Cache;

    /// The timestamp of the last refresh.
    ///
    /// The timestamp is taken from the root directory.
    fn timestamp(&mut self) -> &mut Option<std::time::SystemTime>;

    /// The directory root from which to load items.
    fn root(&self) -> &path::PathBuf;
}

impl<T> super::Source for T
where
    T: FileSystemSource + Sized,
{
    /// Starts this file system source.
    ///
    /// This method will perform a complete recursive scan of the source
    /// directory.
    ///
    /// # Panics
    /// If an item fails to be added.
    fn start(&mut self) {
        self.notify();
    }

    /// Reloads items from the file system if the root directory has been
    /// modified since the last time it was reloaded.
    fn notify(&mut self) {
        if let Ok(timestamp) = self.root().metadata().and_then(|m| m.modified())
        {
            if self.timestamp().map(|t| t < timestamp).unwrap_or(true) {
                self.populate();
                *self.timestamp() = Some(timestamp)
            }
        }
    }
}

impl<P: AsRef<path::Path>> From<P> for data::Item {
    /// Converts as path to an item by simply reading the modification time.
    ///
    /// If the modification time cannot be read, the current time is used.
    ///
    /// # Arguments
    /// *  `source` - The source path.
    fn from(source: P) -> Self {
        let path: &path::Path = source.as_ref();
        data::Item::new(
            path,
            data::timestamp(&path),
            collections::HashSet::new(),
        )
    }
}
