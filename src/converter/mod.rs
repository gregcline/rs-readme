pub mod github_converter;
pub mod offline_converter;

use std::error::Error;
use std::fmt;

use async_trait::async_trait;
use github_converter::GitHubConverter;
use offline_converter::OfflineConverter;

/// Represents an error from the markdown converter.
#[derive(Debug, PartialEq)]
pub enum MarkdownError {
    ConverterUnavailable(String),
}

impl fmt::Display for MarkdownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MarkdownError::ConverterUnavailable(reason) => {
                write!(f, "Could not convert\n{}", reason)
            }
        }
    }
}

impl Error for MarkdownError {}

/// Something that can convert a markdown string to HTML.
#[async_trait]
pub trait MarkdownConverter {
    async fn convert_markdown(&self, md: &str) -> Result<String, MarkdownError>;
}

/// Allows us to use either a GitHub API-based converter or an offline converter
/// through pulldown cmark.
pub enum Converters {
    Github(GitHubConverter),
    Offline(OfflineConverter),
}

#[async_trait]
impl MarkdownConverter for Converters {
    async fn convert_markdown(&self, md: &str) -> Result<String, MarkdownError> {
        match self {
            Converters::Github(converter) => converter.convert_markdown(&md).await,
            Converters::Offline(offline) => offline.convert_markdown(&md).await,
        }
    }
}
