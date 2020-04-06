use async_trait::async_trait;
use log::error;
use surf;

/// Represents an error from the markdown converter.
#[derive(Debug, PartialEq)]
pub enum MarkdownError {
    ConverterUnavailable(String),
}

/// The JSON body to send some text to GitHub's API to be converted from
/// markdown to HTML.
#[derive(Serialize, Deserialize, Debug)]
struct MarkdownRequest {
    /// The text to be converted.
    text: String,

    /// Can be `markdown` or `gfm` for __GitHub Flavored Markdown__.
    /// `gfm` will make links for things like issues and PRs.
    mode: String,

    /// If in `gfm` you need to provide a context repository like `gregcline/rs-readme`.
    context: String,
}

/// Something that can convert a markdown string to HTML.
#[async_trait]
pub trait MarkdownConverter {
    async fn convert_markdown(&self, md: &str) -> Result<String, MarkdownError>;
}

/// Can convert from markdown to HTML using the GitHub API.
pub struct Converter {
    api_path: String,
}

impl Converter {
    /// Builds a new converter using the given GitHub API.
    pub fn new(api_path: String) -> Converter {
        Converter { api_path }
    }
}

#[async_trait]
impl MarkdownConverter for Converter {
    /// Makes a request to the GitHub API and returns the resulting string.
    async fn convert_markdown(&self, md: &str) -> Result<String, MarkdownError> {
        let client = surf::Client::new();

        let mut resp = client
            .post(format!("{}/markdown", &self.api_path))
            .body_json(&MarkdownRequest {
                text: md.to_string(),
                mode: "markdown".to_string(),
                context: "".to_string(),
            })
            .map_err(|err| {
                error!("{:?}", err);
                MarkdownError::ConverterUnavailable("Error making request".to_string())
            })?
            .await
            .map_err(|err| {
                error!("{:?}", err);
                MarkdownError::ConverterUnavailable("Error awaiting response".to_string())
            })?;

        let body = resp
            .body_string()
            .await
            .unwrap_or_else(|_| "Could not read response body from GitHub".to_string());

        if resp.status().as_u16() >= 400 {
            Err(MarkdownError::ConverterUnavailable(body))
        } else {
            Ok(body)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use mockito::{mock, Matcher};

    #[async_std::test]
    async fn converter_makes_proper_web_request() {
        let m = mock("POST", "/markdown")
            .match_body(Matcher::JsonString(
                "{\"text\": \"# A thing!\", \"mode\": \"markdown\", \"context\": \"\"}".to_string(),
            ))
            .with_body("<h1>A thing!</h1>")
            .expect(1)
            .create();

        let converter = Converter::new(mockito::server_url());
        let html = converter.convert_markdown("# A thing!").await;

        m.assert();
        assert_eq!(html, Ok("<h1>A thing!</h1>".to_string()));
    }

    #[async_std::test]
    async fn api_over_400_results_in_converter_unavailable() {
        let m = mock("POST", "/markdown")
            .with_status(400)
            .with_body("Github error message")
            .expect(1)
            .create();

        let converter = Converter::new(mockito::server_url());
        let html = converter.convert_markdown("# A thing!").await;

        m.assert();
        assert_eq!(
            html,
            Err(MarkdownError::ConverterUnavailable(
                "Github error message".to_string()
            ))
        );
    }
}
