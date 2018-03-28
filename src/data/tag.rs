use std::fmt;
use std::str;

/// A tag.
///
/// Tags exist in a tree structure and have a name, a full path and an optional
/// parent.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Tag {
    /// The individual parts of this tag.
    pub parts: Vec<String>,
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
        Self {
            parts: path.split(Self::SEPARATOR).map(String::from).collect(),
        }
    }

    /// Creates a leaf tag.
    ///
    /// `name` is split on separators and each individual part is added.
    ///
    /// # Arguments
    /// *  `parent` - The parent tag.
    /// *  `name` - The name of the leaf tag.
    pub fn leaf(&self, name: &str) -> Self {
        let mut parts = self.parts.clone();
        parts.extend(name.split(Self::SEPARATOR).map(String::from));
        Self { parts }
    }

    /// Returns whether this is a root tag.
    pub fn is_root(&self) -> bool {
        self.parts.len() < 2
    }

    /// Returns whether this tag is a parent of another tag.
    ///
    /// A tag is a parent of another tag if the other tag is more deeply nested
    /// and shares all initial parts with this tag.
    ///
    /// # Arguments
    /// *  `other` - The tag to check.
    pub fn is_parent_of(&self, other: &Self) -> bool {
        other.parts.len() > self.parts.len()
            && self.parts
                .iter()
                .zip(other.parts.iter())
                .all(|(a, b)| a == b)
    }

    /// Returns the leaf name.
    pub fn name(&self) -> Option<&String> {
        self.parts.last()
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (i, part) in self.parts.iter().enumerate() {
            if i == 0 {
                write!(formatter, "{}", part)?;
            } else {
                write!(formatter, "{}{}", Self::SEPARATOR, part)?;
            }
        }
        Ok(())
    }
}

impl str::FromStr for Tag {
    type Err = ();

    /// Converts a string to a tag.
    ///
    /// This function will succeed unless the string is empty, or starts or ends
    /// with a separator.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() == 0 || s.chars().next() == Some(Self::SEPARATOR)
            || s.chars().last() == Some(Self::SEPARATOR)
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

    /// Tests that `leaf` works as expected.
    #[test]
    fn leaf() {
        let root = Tag::new("root");
        assert_eq!("root/leaf", root.leaf("leaf").to_string());
    }

    /// Tests that `is_root` works as expected.
    #[test]
    fn is_root() {
        let root = Tag::new("root");
        assert!(root.is_root());
        assert!(!root.leaf("leaf").is_root());
    }

    /// Test that a tag without parent behaves correctly.
    #[test]
    fn no_parent() {
        assert_eq!("root", Tag::new("root").to_string());
    }

    /// Tests that a tag with parent behaves correctly.
    #[test]
    fn with_parent() {
        let root = Tag::new("root");

        let leaf1 = root.leaf("leaf");
        assert_eq!(Some(&String::from("leaf")), leaf1.name());
        assert_eq!("root/leaf", leaf1.to_string());

        let leaf2 = leaf1.leaf("sub");
        assert_eq!(Some(&String::from("sub")), leaf2.name());
        assert_eq!("root/leaf/sub", leaf2.to_string());
    }

    /// Tests that an invalid string cannot be parsed.
    #[test]
    fn from_str_invalid() {
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
    fn from_str_with_parent() {
        let leaf1 = "root/leaf".parse::<Tag>().unwrap();
        assert_eq!(Some(&String::from("leaf")), leaf1.name());
        assert_eq!("root/leaf", leaf1.to_string());

        let leaf2 = "root/leaf/sub".parse::<Tag>().unwrap();
        assert_eq!(Some(&String::from("sub")), leaf2.name());
        assert_eq!("root/leaf/sub", leaf2.to_string());
    }

    #[test]
    fn is_parent_of() {
        assert!(Tag::new("a").is_parent_of(&Tag::new("a/b")));
        assert!(Tag::new("a").is_parent_of(&Tag::new("a/b/c")));
        assert!(!Tag::new("a/b").is_parent_of(&Tag::new("a/b")));
        assert!(!Tag::new("a/b").is_parent_of(&Tag::new("a")));
        assert!(!Tag::new("b/a").is_parent_of(&Tag::new("a")));
    }
}
