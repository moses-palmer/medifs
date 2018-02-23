use std;
use std::path;

use clap;

use data;
use files;

use super::*;
use sources::*;


file_system_base!(
    DirectorySource,
);


impl FileSystemItemGenerator for DirectorySource {
    /// Generates an item from a path.
    ///
    /// # Arguments
    /// *  `path` - The path for which to generate an item.
    fn item(&self, path: &path::Path) -> data::Item {
        path.into()
    }
}


impl ConfigurableSource for DirectorySource {
    const SUBCOMMAND_NAME: &'static str = "directory";

    fn options<'a>(app: clap::App<'a, 'a>) -> clap::App<'a, 'a> {
        options(app)
    }
}


impl ConstructableSource for DirectorySource {
    fn construct<'a>(
        cache: files::Cache,
        args: &clap::ArgMatches<'a>,
    ) -> Result<Self, String> {
        let root = args.value_of(OPT_ROOT).map(|v| v.into()).unwrap();
        Ok(DirectorySource {
            cache,
            root,
            timestamp: None,
        })
    }
}
