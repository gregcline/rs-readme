use crate::markdown_converter::{MarkdownConverter, MarkdownError};
use async_trait::async_trait;
use pulldown_cmark::{html, Options, Parser};

pub struct OfflineConverter {
    options: Options,
}

impl OfflineConverter {
    /// Builds a new offline converter.
    pub fn new() -> OfflineConverter {
        OfflineConverter {
            options: Options::all(),
        }
    }
}

impl Default for OfflineConverter {
    fn default() -> Self {
        OfflineConverter::new()
    }
}

#[async_trait]
impl MarkdownConverter for OfflineConverter {
    async fn convert_markdown(&self, md: &str) -> Result<String, MarkdownError> {
        let parser = Parser::new_ext(&md, self.options);

        let mut html_output = String::new();

        html::push_html(&mut html_output, parser);

        Ok(html_output)
    }
}
