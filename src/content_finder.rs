use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

pub enum ContentError {
    CouldNotFetch,
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

        let mut file = File::open(path).map_err(|_| ContentError::CouldNotFetch)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|_| ContentError::CouldNotFetch)?;

        Ok(contents)
    }
}
