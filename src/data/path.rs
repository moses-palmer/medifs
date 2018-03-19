use std::cmp;
use std::ffi;
use std::fmt;
use std::path;

use mime;
use mime_guess;


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


/// An item that has a file base.
pub trait FileBase {
    /// The type of the extension.
    type T: fmt::Display;

    /// The base of this item.
    fn file_base(&self) -> Self::T;
}

impl FileBase for ffi::OsString {
    type T = String;

    fn file_base(&self) -> Self::T {
        let path: &path::Path = self.as_ref();
        path.file_stem()
            .and_then(|s| s.to_str())
            .map(String::from)
            .unwrap_or(String::from("bin"))
    }
}


/// An item that has a file extension.
pub trait FileExtension {
    /// The type of the extension.
    type T: fmt::Display;

    /// The extension of this item.
    fn file_extension(&self) -> Self::T;
}

impl FileExtension for ffi::OsString {
    type T = String;

    fn file_extension(&self) -> Self::T {
        let path: &path::Path = self.as_ref();
        path.extension()
            .and_then(|s| s.to_str())
            .map(String::from)
            .unwrap_or(String::from("bin"))
    }
}

impl FileExtension for mime::Mime {
    type T = &'static str;

    fn file_extension(&self) -> Self::T {
        // Unfortunately we cannot rely on mime::guess_mime_type to return the
        // preferred extension, so we explicitly handle JPEG and PNG
        if self == &mime::IMAGE_JPEG {
            &"jpeg"
        } else if self == &mime::IMAGE_PNG {
            &"png"
        } else {
            mime_guess::get_mime_extensions(&self)
                .and_then(|mts| mts.iter().next())
                .unwrap_or(&"bin")
        }
    }
}


/// Constructs a file name from a base, a file type and an index.
///
/// The result will consist of the base joined by the extension if `index` is
/// `0`, otherwise `index` parenthesised will be inserted after the base and a
/// space.
///
/// # Arguments
/// *  `base` - The base name.
/// *  `ext` - The file extension.
/// *  `index` - An index to incorporate into the name in case of multiple
///    items with the same name.
pub fn name<B: FileBase, E: FileExtension>(
    base: &B,
    ext: &E,
    index: usize,
) -> path::PathBuf {
    if index > 0 {
        format!("{} ({}).{}", base.file_base(), index, ext.file_extension())
            .into()
    } else {
        format!("{}.{}", base.file_base(), ext.file_extension()).into()
    }
}


#[cfg(test)]
mod tests {
    use mime;

    use super::*;

    impl FileBase for String {
        type T = String;

        fn file_base(&self) -> Self::T {
            self.clone()
        }
    }


    /// Tests that the name is generated as expected.
    #[test]
    fn name_correct() {
        assert_eq!(
            path::PathBuf::from("test1.jpeg"),
            name(&String::from("test1"), &mime::IMAGE_JPEG, 0),
        );
        assert_eq!(
            path::PathBuf::from("test2 (1).jpeg"),
            name(&String::from("test2"), &mime::IMAGE_JPEG, 1),
        );
        assert_eq!(
            path::PathBuf::from("test3 (2).png"),
            name(&String::from("test3"), &mime::IMAGE_PNG, 2),
        );
    }
}
