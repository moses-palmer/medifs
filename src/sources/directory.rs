use std;
use std::collections;
use std::path;

use clap;
use time;

use data;
use files;

use super::ConfigurableSource;
use super::file_system::{FileSystemSource, OPT_ROOT, options};


pub struct DirectorySource {
    /// The root directory.
    root: path::PathBuf,

    /// The cache.
    cache: files::Cache,

    /// The timestamp of last refresh.
    timestamp: Option<std::time::SystemTime>,
}


impl FileSystemSource for DirectorySource {
    /// Generates an item from a path.
    ///
    /// # Arguments
    /// *  `path` - The path for which to generate an item.
    fn item<P: AsRef<path::Path>>(&self, path: P) -> data::Item {
        let path: &path::Path = path.as_ref();
        data::Item::new(
            path,
            time::at(
                path.metadata()
                    .and_then(|meta| meta.modified())
                    .unwrap_or(std::time::SystemTime::now())
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| {
                        time::Timespec::new(
                            d.as_secs() as i64,
                            d.subsec_nanos() as i32,
                        )
                    })
                    .unwrap_or(time::Timespec::new(0, 0)),
            ),
            collections::HashSet::new(),
        )
    }

    /// The cache.
    ///
    /// This value will not be available until this source has been started.
    #[inline(always)]
    fn cache(&self) -> &files::Cache {
        &self.cache
    }

    /// The timestamp of the last refresh.
    ///
    /// The timestamp is taken from the root directory modification time.
    #[inline(always)]
    fn timestamp(&mut self) -> &mut Option<std::time::SystemTime> {
        &mut self.timestamp
    }

    /// The directory root from which to load items.
    #[inline(always)]
    fn root(&self) -> &path::PathBuf {
        &self.root
    }
}


impl ConfigurableSource for DirectorySource {
    const SUBCOMMAND_NAME: &'static str = "directory";

    fn options<'a>(app: clap::App<'a, 'a>) -> clap::App<'a, 'a> {
        options(app)
    }

    fn construct<'a>(
        cache: files::Cache,
        args: &clap::ArgMatches<'a>,
    ) -> Result<Self, String> {
        Ok(DirectorySource {
            cache,
            root: args.value_of(OPT_ROOT).map(|v| v.into()).unwrap(),
            timestamp: None,
        })
    }
}
