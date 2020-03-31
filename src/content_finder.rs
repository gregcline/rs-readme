use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

use log::{error, warn};

#[derive(Debug, PartialEq)]
pub enum ContentError {
    CouldNotFetch,
    NotMarkdown,
}

pub trait ContentFinder {
    fn content_for(&self, resource: &str) -> Result<String, ContentError>;
}

pub struct Finder {
    root: PathBuf,
}

impl Finder {
    pub fn new(root: PathBuf) -> Finder {
        Finder { root }
    }
}

impl ContentFinder for Finder {
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
        file.read_to_string(&mut contents)
            .map_err(|err| {
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
        let finder = Finder::new(PathBuf::from("./"));

        let content_for_a = finder.content_for("test_dir/a.md");
        let content_for_b = finder.content_for("test_dir/b.md");

        assert_eq!(content_for_a, Ok("# A's content\n".to_string()));
        assert_eq!(content_for_b, Ok("- B's content\n".to_string()));
    }

    #[test]
    fn does_not_find_content_in_txt() {
        let finder = Finder::new(PathBuf::from("./"));

        let err = finder.content_for("test_dir/b.txt");

        assert_eq!(err, Err(ContentError::NotMarkdown));
    }
}
