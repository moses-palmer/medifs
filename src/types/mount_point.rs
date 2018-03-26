use std::path;
use std::str;

use clap;

pub struct MountPoint(path::PathBuf);

impl str::FromStr for MountPoint {
    type Err = clap::Error;

    fn from_str(source: &str) -> Result<Self, Self::Err> {
        let path = path::PathBuf::from(source);
        if !path.is_dir() {
            Err(clap::Error::with_description(
                format!("{} is not a directory", source).as_str(),
                clap::ErrorKind::InvalidValue,
            ))
        } else if let Ok(dir) = path.read_dir() {
            if dir.count() == 0 {
                Ok(MountPoint(path))
            } else {
                Err(clap::Error::with_description(
                    format!("{} is not an empty directory", source).as_str(),
                    clap::ErrorKind::InvalidValue,
                ))
            }
        } else {
            Err(clap::Error::with_description(
                format!("failed to read {}", source).as_str(),
                clap::ErrorKind::InvalidValue,
            ))
        }
    }
}

impl AsRef<path::Path> for MountPoint {
    fn as_ref(&self) -> &path::Path {
        &self.0
    }
}
