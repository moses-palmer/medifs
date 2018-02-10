use clap;

use files;

mod file_system;

pub mod directory;
pub use self::directory::DirectorySource;


/// A source of media files.
pub trait Source: Send + Sync {
    /// Starts this source.
    ///
    /// This method should load all items from this source into the cache.
    fn start(&mut self);

    /// Notifies this source that a file system access is being attempted.
    ///
    /// This method should check whether the underlying data store has been
    /// updated, and in that case update the cache.
    fn notify(&mut self);
}


/// A source of media files.
pub trait ConfigurableSource: Source + Sized {
    /// The name of the command line subcommand.
    const SUBCOMMAND_NAME: &'static str;

    /// Generates a description of the command line argument group used to
    /// configure this source.
    fn options<'a>(app: clap::App<'a, 'a>) -> clap::App<'a, 'a>;

    /// Constructs a source from command line arguments.
    fn construct<'a>(
        cache: files::Cache,
        args: &clap::ArgMatches<'a>,
    ) -> Result<Self, String>;
}


/// An object with sources.
pub trait WithSources<'a>: Sized {
    /// Applies all source to this instance.
    ///
    /// This is where to add new sources.
    fn with_sources(self) -> Self {
        self.with_source::<DirectorySource>()
    }

    /// Applies a single source to this.
    fn with_source<S: ConfigurableSource>(self) -> Self;
}


impl<'a> From<(files::Cache, clap::ArgMatches<'a>)> for Box<Source> {
    /// Converts a cache and arguments to a boxed source.
    ///
    /// This is where to add new sources.
    ///
    /// # Arguments
    /// *  `cache` - The cache to which to add items.
    /// *  `matches` - Command line arguments.
    fn from((cache, args): (files::Cache, clap::ArgMatches<'a>)) -> Self {
        match args.subcommand() {
            (DirectorySource::SUBCOMMAND_NAME, Some(ref app)) => {
                DirectorySource::construct(cache, app)
                    .map(|s| Box::new(s))
                    .expect("failed to construct directory source")
            }
            _ => panic!("no source specified"),
        }
    }
}


impl<'a> WithSources<'a> for clap::App<'a, 'a> {
    fn with_source<S: ConfigurableSource>(self) -> Self {
        self.subcommand(
            S::options(clap::SubCommand::with_name(S::SUBCOMMAND_NAME)),
        )
    }
}
