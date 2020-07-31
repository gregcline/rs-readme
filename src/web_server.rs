use async_trait::async_trait;
use horrorshow::helper::doctype;
use horrorshow::prelude::*;
use http_types::mime;
use serde_json::json;
use std::sync::Arc;
use tide::{
    http::StatusCode, log, sse::Sender, Middleware, Next, Request, Response, Server, Status,
};

use crate::content_finder::{ContentError, ContentFinder};
use crate::markdown_converter::{Converter, MarkdownConverter, MarkdownError};
use crate::offline_converter::OfflineConverter;
use crate::static_files;

/// Allows us to use either a GitHub API-based converter or an offline converter
/// through pulldown cmark.
pub enum Converters {
    Github(Converter),
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

/// The state necessary to process requests.
///
/// It needs something to find some markdown content based on a URL path and something to take that
/// markdown content and convert it to HTML to display.
pub struct State<M, C>
where
    M: MarkdownConverter,
    C: ContentFinder,
{
    markdown_converter: M,
    content_finder: C,
}

impl<M, C> State<M, C>
where
    M: MarkdownConverter + Send + Sync + 'static,
    C: ContentFinder + Send + Sync + 'static,
{
    pub fn new(markdown_converter: M, content_finder: C) -> State<M, C> {
        State {
            markdown_converter,
            content_finder,
        }
    }
}

/// The basic HTML of our page, the `<head>` and CSS and `<body>`.
/// Also includes the script to subscribe to the Server Sent Events for the page
/// and update the page if the file changes.
fn base_html(title: &str, content: &str) -> String {
    format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    link(rel="stylesheet", href="/static/octicons/octicons.css");
                    link(rel="stylesheet", href="https://github.githubassets.com/assets/frameworks-146fab5ea30e8afac08dd11013bb4ee0.css");
                    link(rel="stylesheet", href="https://github.githubassets.com/assets/site-897ad5fdbe32a5cd67af5d1bdc68a292.css");
                    link(rel="stylesheet", href="https://github.githubassets.com/assets/github-c21b6bf71617eeeb67a56b0d48b5bb5c.css");
                    link(rel="stylesheet", href="/static/style.css");
                    title : title;
                    script {
                        : Raw("let hash = '';
                           let event = new EventSource(`//${location.host}/__rs-readme${location.pathname}`);
                           event.addEventListener('update', (e) => {
                              let message = JSON.parse(e.data);
                              if (message.hash !== hash) {
                                  hash = message.hash;
                                  document.getElementById('rs-readme-content').innerHTML = message.contents;
                              }
                           });")
                    }
                }
                body : Raw(content);
            }
        }
    )
}

/// The wrapping necessary to make the rendered markdown file to look right
fn markdown_html(file_name: &str, md_content: &str) -> String {
    format!(
        "{}",
        html! {
            div(class="page") {
                div(id="preview-page", class="preview-page") {
                    div(role="main", class="main-content") {
                        div(class="container new-discussion-timeline experiment-repo-nav") {
                            div(class="repository-content") {
                                div(id="readme", class="readme boxed-group clearfix announce instapaper_body md") {
                                    h3 {
                                        span(class="octicon octicon-book");
                                        : format!(" {}",file_name);
                                    }
                                    article(id="rs-readme-content", class="markdown-body entry-content", itemprop="text") {
                                        : Raw(md_content);
                                    }
                                }
                            }
                        }
                    }
                }
                div : Raw("&nbsp;");
            }
        }
    )
}

/// The error HTML indicating the requested file is not markdown
/// and therefore can't be rendered.
fn not_markdown_html(title: &str, file: &str) -> String {
    format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    title : title;
                }
                body {
                    h1 : "Not a Markdown File";
                    p {
                        strong : file;
                        : " is not a markdown file and cannot be rendered";
                    }
                }
        }}
    )
}

/// The error HTML indicating the requested file cannot be found.
fn file_not_found(title: &str, file: &str) -> String {
    format!(
        "{}",
        html! {
            : doctype::HTML;
            html {
                head {
                    title : title;
                }
                body {
                    h1 {
                        : "Couldn't find ";
                        : file;
                    }
                     p {
                         : "For the index page ";
                         em : "rs-readme";
                         : " will look for a file named README in the root folder. Otherwise it looks for an exact file name.";
                     }

                }
            }
        }
    )
}

/// The `tide::Endpoint` to render the `README.md`.
///
/// It assumes that there will be a `README.md` in your folder. It lets us have a special error
/// message for it and lets the root of the website render `README.md`. It might not be necessary
/// though, maybe we could just redirect `/` to `/README.md`.
async fn render_readme(
    req: Request<
        Arc<
            State<impl MarkdownConverter + Send + Sync + 'static, impl ContentFinder + Send + Sync>,
        >,
    >,
) -> tide::Result {
    let state = req.state();

    let (contents, _hash) = state
        .content_finder
        .content_for("README.md")
        .with_status(|| StatusCode::NotFound)?;

    let converted = state.markdown_converter.convert_markdown(&contents).await?;

    let resp = base_html("README.md", &markdown_html("README.md", &converted));

    Ok(Response::builder(StatusCode::Ok)
        .body(resp)
        .content_type(mime::HTML)
        .build())
}

/// Renders any given file path containing markdown as HTML.
async fn render_markdown_path(
    req: Request<
        Arc<
            State<impl MarkdownConverter + Send + Sync + 'static, impl ContentFinder + Send + Sync>,
        >,
    >,
) -> tide::Result {
    let state = req.state();

    let path = req.url().path();
    let file = path.split('/').last().unwrap_or("rs-readme");

    let (contents, _hash) = state.content_finder.content_for(&format!(".{}", path))?;

    let converted = state.markdown_converter.convert_markdown(&contents).await?;

    let resp = base_html(file, &markdown_html(file, &converted));

    Ok(Response::builder(StatusCode::Ok)
        .body(resp)
        .content_type(mime::HTML)
        .build())
}

/// Sends an event periodically with the file contents and the SHA1 of the contents.
/// The front end will update if the hash differs.
async fn render_page_update(
    req: Request<
        Arc<State<impl MarkdownConverter + Send + Sync, impl ContentFinder + Send + Sync>>,
    >,
    sender: Sender,
) -> Result<(), http_types::Error> {
    let state = req.state();

    let path = &req.url().path()["/__rs-readme".len()..];
    let (contents, hash) = if path == "/" {
        state.content_finder.content_for("./README.md")?
    } else {
        state.content_finder.content_for(&format!(".{}", path))?
    };

    let converted = state.markdown_converter.convert_markdown(&contents).await?;

    let message = json!({
        "contents": &converted,
        "hash": &format!("{:x}", &hash),
    });

    sender.send("update", &message.to_string(), None).await?;

    Ok(())
}

struct ErrorMiddleware {}

impl ErrorMiddleware {
    fn not_markdown(&self, path: &str) -> tide::Result {
        Ok(Response::builder(StatusCode::BadRequest)
            .body(not_markdown_html("rs-readme", path))
            .content_type(mime::HTML)
            .build())
    }

    fn not_found(&self, resource: &str) -> tide::Result {
        Ok(Response::builder(StatusCode::NotFound)
            .body(file_not_found("rs-readme", resource))
            .content_type(mime::HTML)
            .build())
    }
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for ErrorMiddleware {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let url = req.url().clone();
        let res = next.run(req).await;
        if let Some(err) = res.downcast_error::<ContentError>() {
            match err {
                ContentError::NotMarkdown => self.not_markdown(url.path()),
                ContentError::CouldNotFetch(resource) => self.not_found(resource),
            }
        } else {
            Ok(res)
        }
    }
}

/// Builds a `tide::Server` with the appropriate endpoint mappings.
pub fn build_app(
    state: Arc<
        State<
            impl MarkdownConverter + Send + Sync + 'static,
            impl ContentFinder + Send + Sync + 'static,
        >,
    >,
) -> Server<Arc<State<impl MarkdownConverter, impl ContentFinder>>> {
    let mut app = Server::with_state(state);
    app.with(log::LogMiddleware::new());
    app.with(ErrorMiddleware {});
    app.at("").get(render_readme);
    app.at("/static/octicons/:file").get(static_files::octicons);
    app.at("/static/style.css").get(static_files::style);
    app.at("/__rs-readme/")
        .get(tide::sse::endpoint(render_page_update));
    app.at("/__rs-readme/*")
        .get(tide::sse::endpoint(render_page_update));
    app.at("/*").get(render_markdown_path);

    app
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_base_html() {
        let expected = "\
<!DOCTYPE html>\
<html>\
  <head>\
  <link rel=\"stylesheet\" href=\"/static/octicons/octicons.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/frameworks-146fab5ea30e8afac08dd11013bb4ee0.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/site-897ad5fdbe32a5cd67af5d1bdc68a292.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/github-c21b6bf71617eeeb67a56b0d48b5bb5c.css\">\
  <link rel=\"stylesheet\" href=\"/static/style.css\">\
    <title>test title</title>\
    <script>let hash = '';
                           let event = new EventSource(`//${location.host}/__rs-readme${location.pathname}`);
                           event.addEventListener('update', (e) => {
                              let message = JSON.parse(e.data);
                              if (message.hash !== hash) {
                                  hash = message.hash;
                                  document.getElementById('rs-readme-content').innerHTML = message.contents;
                              }
                           });</script>\
  </head>\
  <body>\
    Test content\
  </body>\
</html>";

        let actual = base_html("test title", "Test content");

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_markdown_html() {
        let expected = "\
<div class=\"page\">\
  <div id=\"preview-page\" class=\"preview-page\">\
    <div role=\"main\" class=\"main-content\">\
      <div class=\"container new-discussion-timeline experiment-repo-nav\">\
        <div class=\"repository-content\">\
          <div id=\"readme\" class=\"readme boxed-group clearfix announce instapaper_body md\">\
            <h3>\
              <span class=\"octicon octicon-book\"></span> \
              file_name.md\
            </h3>\
            <article id=\"rs-readme-content\" class=\"markdown-body entry-content\" itemprop=\"text\">\
              Test content\
            </article>\
          </div>\
        </div>\
      </div>\
    </div>\
  </div>\
  <div>&nbsp;</div>\
</div>";

        let actual = markdown_html("file_name.md", "Test content");

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_not_markdown_html() {
        let expected = "\
<!DOCTYPE html>\
<html>\
<head><title>rs-readme</title></head>\
<body>\
<h1>Not a Markdown File</h1>\
<p><strong>test_file</strong> is not a markdown file and cannot be rendered</p>\
</body>\
</html>\
";

        let actual = not_markdown_html("rs-readme", "test_file");

        assert_eq!(expected, actual);
    }
}
