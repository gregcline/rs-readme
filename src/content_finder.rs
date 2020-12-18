use std::error::Error;
use std::ffi::OsStr;
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use generic_array::{typenum::U20, GenericArray};
use log::{error, warn};
use sha1::{Digest, Sha1};

/// The possible errors while finding some markdown content.
///
/// There are a lot of possible file system errors that I just
/// don't want/need to worry about right now all encapsulated
/// in `CouldNotFetch`.
#[derive(Debug, PartialEq)]
pub enum ContentError {
    /// The requested content couldn't be fetched (probably a file error)
    CouldNotFetch(String),

    /// The requested content wasn't markdown
    NotMarkdown,
}

impl fmt::Display for ContentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContentError::CouldNotFetch(resource) => {
                write!(f, "Could not find {}", resource.replacen("./", "", 1))
            }
            ContentError::NotMarkdown => write!(f, "The file was not markdown"),
        }
    }
}

impl Error for ContentError {}

/// Something that can find some markdown content given a resource identifier.
pub trait ContentFinder {
    /// Given a resource identifier returns the markdown string it represents.
    fn content_for(&self, resource: &str) -> Result<(String, GenericArray<u8, U20>), ContentError>;
}

/// Implements [`ContentFinder`] based on a file folder.
///
/// It expects any `resource` to be a valid file path and will look for that
/// path relative to `root`. If it finds a markdown file it returns its
/// contents, otherwise it returns an error.
pub struct FileFinder {
    root: PathBuf,
}

impl FileFinder {
    /// Creates a new [`FileFinder`] relative to `root`.
    pub fn new(root: PathBuf) -> FileFinder {
        FileFinder { root }
    }
}

impl ContentFinder for FileFinder {
    /// Returns the contents of the file located at the path in `resource`.
    fn content_for(&self, resource: &str) -> Result<(String, GenericArray<u8, U20>), ContentError> {
        let mut path = self.root.clone();
        path.push(resource);

        if path.extension().unwrap_or_else(|| OsStr::new("")) != "md" {
            warn!(
                "Tried to fetch markdown from {}, please add .md extension",
                path.to_string_lossy()
            );
            return Err(ContentError::NotMarkdown);
        }

        let mut file = File::open(&path).map_err(|err| {
            error!(
                "Could not open file {}:\n{:#?}",
                path.to_string_lossy(),
                err
            );
            ContentError::CouldNotFetch(resource.to_string())
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|err| {
            error!(
                "Could not read contents of {}:\n{:#?}",
                path.to_string_lossy(),
                err
            );
            ContentError::CouldNotFetch(resource.to_string())
        })?;

        let hash = Sha1::digest(&contents.as_bytes());

        Ok((contents, hash))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn finds_content_in_md() {
        let finder = FileFinder::new(PathBuf::from("./"));

        let (content_for_a, hash_for_a) = finder.content_for("test_dir/a.md").unwrap();
        let (content_for_b, hash_for_b) = finder.content_for("test_dir/b.md").unwrap();

        #[cfg(not(windows))]
        let a_expected = "# A's content\n![](./images/rust-logo.png)\n".to_string();
        #[cfg(windows)]
        let a_expected = "# A's content\r\n![](./images/rust-logo.png)\r\n".to_string();
        #[cfg(not(windows))]
        let b_expected = "- B's content\n".to_string();
        #[cfg(windows)]
        let b_expected = "- B's content\r\n".to_string();

        assert_eq!(content_for_a, a_expected);
        assert_eq!(content_for_b, b_expected);
        assert_eq!(hash_for_a, Sha1::digest(a_expected.as_bytes()));
        assert_eq!(hash_for_b, Sha1::digest(b_expected.as_bytes()));
    }

    #[test]
    fn does_not_find_content_in_txt() {
        let finder = FileFinder::new(PathBuf::from("./"));

        let err = finder.content_for("test_dir/b.txt");

        assert_eq!(err, Err(ContentError::NotMarkdown));
    }
}
