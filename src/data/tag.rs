use std::fmt;
use std::str;


/// A tag.
///
/// Tags exist in a tree structure and have a name, a full path and an optional
/// parent.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Tag {
    /// The name of the parent tag.
    pub parent: Option<String>,

    /// The last part of the tag name.
    pub leaf: String,
}

impl Tag {
    const SEPARATOR: char = '/';

    /// Creates a named tag.
    ///
    /// # Arguments
    /// *  `path` - The full name of the tag.
    ///
    ///    This value may contain any number of separators.
    pub fn new(path: &str) -> Self {
        let (parent, leaf) = Self::split(path);
        Self { parent, leaf }
    }

    /// Creates a root tag.
    ///
    /// A root tag may not contain any separators.
    ///
    /// # Arguments
    /// *  `name` - The name of the root tag.
    pub fn root(name: &str) -> Option<Self> {
        if Self::is_flat(name) {
            Some(Self {
                parent: None,
                leaf: name.to_string(),
            })
        } else {
            None
        }
    }

    /// Creates a leaf tag.
    ///
    /// A the name of a leaf tag may not contain any separators.
    ///
    /// # Arguments
    /// *  `parent` - The parent tag.
    /// *  `name` - The name of the leaf tag.
    pub fn leaf(parent: &Tag, name: &str) -> Option<Self> {
        if Self::is_flat(name) {
            Some(Self {
                parent: Some(parent.to_string()),
                leaf: name.to_string(),
            })
        } else {
            None
        }
    }

    /// Returns whether this is a root tag.
    pub fn is_root(self) -> bool {
        self.parent.is_none()
    }

    /// Splits a string into possibly the parent part and the final part.
    ///
    /// A string with no separators will yield no parent part. An empty string
    /// will yield no parent part and an empty name part.
    ///
    /// # Arguments
    /// *  `name` - The string to split.
    fn split(name: &str) -> (Option<String>, String) {
        if let Some(index) = name.rfind(Self::SEPARATOR) {
            let (parent, leaf) = name.split_at(index);
            (Some(parent.to_string()), leaf[1..].to_string())
        } else {
            (None, name.to_string())
        }
    }

    /// Determines whether a tag name is flat.
    ///
    /// A flat name does not contain any separatos.
    ///
    /// # Arguments
    /// *  `name` - The name of the tag.
    fn is_flat(name: &str) -> bool {
        !name.contains(Self::SEPARATOR)
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        if let Some(ref parent) = self.parent {
            write!(formatter, "{}{}{}", parent, Self::SEPARATOR, self.leaf)
        } else {
            write!(formatter, "{}", self.leaf)
        }
    }
}

impl str::FromStr for Tag {
    type Err = ();

    /// Converts a string to a tag.
    ///
    /// This function will succeed unless the string is empty, or starts or ends
    /// with a separator.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 0 || s.chars().next() == Some(Self::SEPARATOR) ||
            s.chars().last() == Some(Self::SEPARATOR)
        {
            Err(())
        } else {
            Ok(Self::new(s))
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    /// Tests that `root` works as expected.
    #[test]
    fn root() {
        assert!(Tag::root("root").is_some());
        assert_eq!("root", Tag::root("root").unwrap().to_string());

        assert!(Tag::root("root/leaf").is_none());
    }

    /// Tests that `leaf` works as expected.
    #[test]
    fn leaf() {
        let root = Tag::new("root");

        assert!(Tag::leaf(&root, "leaf").is_some());
        assert_eq!("root/leaf", Tag::leaf(&root, "leaf").unwrap().to_string());

        assert!(Tag::leaf(&root, "leaf/sub").is_none());
    }

    /// Tests that `is_root` works as expected.
    #[test]
    fn is_root() {
        assert!(Tag::root("root").unwrap().is_root());

        let root = Tag::root("root").unwrap();
        assert!(!Tag::leaf(&root, "leaf").unwrap().is_root());
    }

    /// Test that a tag without parent behaves correctly.
    #[test]
    fn no_parent() {
        assert_eq!("root", Tag::root("root").unwrap().to_string());
        assert_eq!("root", Tag::new("root").to_string());
    }

    /// Tests that a tag with parent behaves correctly.
    #[test]
    fn with_parent() {
        let root = Tag::new("root");

        let leaf1 = Tag::leaf(&root, "leaf").unwrap();
        assert_eq!(Some("root".to_string()), leaf1.parent);
        assert_eq!("leaf", leaf1.leaf);
        assert_eq!("root/leaf", leaf1.to_string());

        let leaf2 = Tag::leaf(&leaf1, "sub").unwrap();
        assert_eq!(Some("root/leaf".to_string()), leaf2.parent);
        assert_eq!("sub", leaf2.leaf);
        assert_eq!("root/leaf/sub", leaf2.to_string());
    }

    /// Tests that an invalid string cannot be parsed.
    #[test]
    fn from_string_invalid() {
        assert_eq!(Err(()), "".parse::<Tag>());
        assert_eq!(Err(()), "/starts".parse::<Tag>());
        assert_eq!(Err(()), "ends/".parse::<Tag>());
    }

    /// Tests parsing of flat tags.
    #[test]
    fn from_str_no_parent() {
        let tag = "root".parse::<Tag>().unwrap();
        assert_eq!("root", tag.to_string());
    }

    /// Tests parsing of nested tags.
    #[test]
    fn from_string_with_parent() {
        let leaf1 = "root/leaf".parse::<Tag>().unwrap();
        assert_eq!(Some("root".to_string()), leaf1.parent);
        assert_eq!("leaf", leaf1.leaf);
        assert_eq!("root/leaf", leaf1.to_string());

        let leaf2 = "root/leaf/sub".parse::<Tag>().unwrap();
        assert_eq!(Some("root/leaf".to_string()), leaf2.parent);
        assert_eq!("sub", leaf2.leaf);
        assert_eq!("root/leaf/sub", leaf2.to_string());
    }
}
