use std;
use std::path;

use clap;
use mime_guess;
use walkdir;

use data;
use files;


/// The name of the argument specifying the root.
pub const OPT_ROOT: &'static str = &"ROOT";


/// Adds the base options for a file system source.
///
/// # Arguments
/// *  `app` - The application to which to add the arguments.
pub fn options<'a>(app: clap::App<'a, 'a>) -> clap::App<'a, 'a> {
    app.arg(
        clap::Arg::with_name(OPT_ROOT)
            .help("The source directory.")
            .required(true),
    )
}


pub trait FileSystemSource: super::Source {
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
                            mime_guess::guess_mime_type(e.path()).type_() ==
                                "image"
                        })
                        .map(|e| self.item(e.path())),
                )
                .unwrap();
        }
    }

    /// Generates an item from a path.
    ///
    /// # Arguments
    /// *  `path` - The path for which to generate an item.
    fn item<P: AsRef<path::Path>>(&self, path: P) -> data::Item;

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
        if let Ok(timestamp) = self.root().metadata().and_then(
            |m| m.modified(),
        )
        {
            if self.timestamp().map(|t| t < timestamp).unwrap_or(true) {
                self.populate();
                *self.timestamp() = Some(timestamp)
            }
        }
    }
}
