use std::sync::Arc;
use horrorshow::helper::doctype;
use horrorshow::prelude::*;
use http_types::mime;
use tide;
use tide::{http::StatusCode, log, Request, Response, Server, Status};
use tide::utils::After;

use crate::content_finder::{ContentError, ContentFinder};
use crate::markdown_converter::MarkdownConverter;
use crate::static_files;

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

/// The basic HTML of our page, the `<head>` and CSS and `<body>`
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
                                    article(class="markdown-body entry-content", itemprop="text") {
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
fn not_markdown_html(file: &str) -> String {
    format!(
        "{}",
        html! {
            h1 : "Not a Markdown File";
            p {
                strong : file;
                : " is not a markdown file and cannot be rendered";
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
    req: Request<Arc<State<impl MarkdownConverter + Send + Sync, impl ContentFinder + Send + Sync>>>,
) -> tide::Result {
    let state = req.state();

    let contents = state
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
    req: Request<Arc<State<impl MarkdownConverter + Send + Sync, impl ContentFinder + Send + Sync>>>,
) -> tide::Result {
    let state = req.state();

    let path = req.url().path();
    let file = path.split('/').last().unwrap_or("rs-readme");

    let contents = match state.content_finder.content_for(&format!(".{}", path)) {
        Ok(contents) => contents,
        Err(ContentError::NotMarkdown) => {
            return Ok(Response::builder(StatusCode::BadRequest)
                .body(base_html(
                    "rs-readme",
                    &not_markdown_html(&req.url().path()),
                ))
                .content_type(mime::HTML)
                .build())
        }
        Err(ContentError::CouldNotFetch(resource)) => {
            return Ok(Response::builder(StatusCode::NotFound)
                .body(format!("{}", ContentError::CouldNotFetch(resource)))
                .content_type(mime::HTML)
                .build())
        }
    };

    let converted = state.markdown_converter.convert_markdown(&contents).await?;

    let resp = base_html(file, &markdown_html(file, &converted));

    Ok(Response::builder(StatusCode::Ok)
        .body(resp)
        .content_type(mime::HTML)
        .build())
}

/// Builds a `tide::Server` with the appropriate endpoint mappings.
pub fn build_app(
    state: Arc<State<
        impl MarkdownConverter + Send + Sync + 'static,
        impl ContentFinder + Send + Sync + 'static,
    >>,
) -> Server<Arc<State<impl MarkdownConverter, impl ContentFinder>>> {
    let mut app = Server::with_state(state);
    app.middleware(log::LogMiddleware::new());
    app.middleware(After(|mut res: Response| async move {
        if let Some(_) = res.downcast_error::<ContentError>() {
            res.set_status(StatusCode::NotFound);
            res.set_content_type(mime::HTML);
            res.set_body("Could not find README.md");
        }
        Ok(res)
    }));
    app.at("").get(render_readme);
    app.at("/static/octicons/:file").get(static_files::octicons);
    app.at("/static/style.css").get(static_files::style);
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
            <article class=\"markdown-body entry-content\" itemprop=\"text\">\
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
<h1>Not a Markdown File</h1>\
<p><strong>test_file</strong> is not a markdown file and cannot be rendered</p>\
";

        let actual = not_markdown_html("test_file");

        assert_eq!(expected, actual);
    }
}
