use std::cmp;
use std::ffi;
use std::path;


/// A path.
#[derive(Clone, Debug, PartialEq)]
pub struct Path(path::PathBuf);

impl From<ffi::OsString> for Path {
    fn from(source: ffi::OsString) -> Self {
        Path(source.into())
    }
}

impl<'a> From<&'a ffi::OsStr> for Path {
    fn from(source: &'a ffi::OsStr) -> Self {
        Path::from(source.to_os_string())
    }
}

impl<'a> From<&'a path::Path> for Path {
    fn from(source: &'a path::Path) -> Self {
        Path(source.to_path_buf())
    }
}

impl AsRef<path::Path> for Path {
    fn as_ref(&self) -> &path::Path {
        self.0.as_ref()
    }
}

impl Eq for Path {}

impl PartialOrd for Path {
    fn partial_cmp(&self, other: &Path) -> Option<cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Path {
    fn cmp(&self, other: &Path) -> cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
