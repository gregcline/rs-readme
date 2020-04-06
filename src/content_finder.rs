use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use log::{error, warn};

/// The possible errors while finding some markdown content.
///
/// There are a lot of possible file system errors that I just
/// don't want/need to worry about right now all encapsulated
/// in `CouldNotFetch`.
#[derive(Debug, PartialEq)]
pub enum ContentError {
    /// The requested content couldn't be fetched (probably a file error)
    CouldNotFetch,

    /// The requested content wasn't markdown
    NotMarkdown,
}

/// Something that can find some markdown content given a resource identifier.
pub trait ContentFinder {
    /// Given a resource identifier returns the markdown string it represents.
    fn content_for(&self, resource: &str) -> Result<String, ContentError>;
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
    fn content_for(&self, resource: &str) -> Result<String, ContentError> {
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
            ContentError::CouldNotFetch
        })?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).map_err(|err| {
            error!(
                "Could not read contents of {}:\n{:#?}",
                path.to_string_lossy(),
                err
            );
            ContentError::CouldNotFetch
        })?;

        Ok(contents)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn finds_content_in_md() {
        let finder = FileFinder::new(PathBuf::from("./"));

        let content_for_a = finder.content_for("test_dir/a.md");
        let content_for_b = finder.content_for("test_dir/b.md");

        assert_eq!(content_for_a, Ok("# A's content\n".to_string()));
        assert_eq!(content_for_b, Ok("- B's content\n".to_string()));
    }

    #[test]
    fn does_not_find_content_in_txt() {
        let finder = FileFinder::new(PathBuf::from("./"));

        let err = finder.content_for("test_dir/b.txt");

        assert_eq!(err, Err(ContentError::NotMarkdown));
    }
}
