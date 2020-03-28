use anyhow::Context;
use async_trait::async_trait;
use surf;

pub enum MarkdownError {
    CouldNotConvert,
}

#[derive(Serialize, Deserialize, Debug)]
struct MarkdownRequest {
    text: String,
    mode: String,
    context: String,
}

#[async_trait]
pub trait MarkdownConverter {
    async fn convert_markdown(&self, md: &str) -> Result<String, MarkdownError>;
}

pub struct Converter {
    api_path: String,
}

impl Converter {
    pub fn new(api_path: String) -> Converter {
        Converter { api_path }
    }
}

#[async_trait]
impl MarkdownConverter for Converter {
    async fn convert_markdown(&self, md: &str) -> Result<String, MarkdownError> {
        let client = surf::Client::new();

        client
            .post(format!("{}/markdown", &self.api_path))
            .body_json(&MarkdownRequest {
                text: md.to_string(),
                mode: "markdown".to_string(),
                context: "".to_string(),
            })
            .with_context(|| "Made an unsuccessful call to the GitHub API")
            .map_err(|_| MarkdownError::CouldNotConvert)?
            .recv_string()
            .await
            .map_err(|_| MarkdownError::CouldNotConvert)
    }
}
