use horrorshow::helper::doctype;
use horrorshow::prelude::*;
use tide;
use tide::{middleware, Request, Response, Server};

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
fn base_html(converted: String) -> String {
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
                    title : "readme-rs";
                }
                body {
                    : Raw(&converted);
                }
            }
        }
    )
}

/// The error HTML indicating the requested file is not markdown
/// and therefore can't be rendered.
fn not_markdown(file: &str) -> String {
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
    req: Request<State<impl MarkdownConverter + Send + Sync, impl ContentFinder + Send + Sync>>,
) -> tide::Result {
    let state = req.state();

    let contents = state
        .content_finder
        .content_for("README.md")
        .map_err(|_| Response::new(404).body_string("Could not find README.md".to_string()))?;

    let converted = state
        .markdown_converter
        .convert_markdown(&contents)
        .await
        .map_err(|_err| {
            Response::new(500).body_string(format!(
                "Could not convert the following markdown:\n {}",
                &contents
            ))
        })?;

    let resp = base_html(converted);

    Ok(Response::new(200)
        .body_string(resp)
        .set_mime(mime::TEXT_HTML_UTF_8))
}

/// Renders any given file path containing markdown as HTML.
async fn render_markdown_path(
    req: Request<State<impl MarkdownConverter + Send + Sync, impl ContentFinder + Send + Sync>>,
) -> tide::Result {
    let state = req.state();

    let contents = state
        .content_finder
        .content_for(&format!(".{}", req.uri().path()))
        .map_err(|err| match err {
            ContentError::NotMarkdown => Response::new(400)
                .body_string(base_html(not_markdown(req.uri().path())))
                .set_mime(mime::TEXT_HTML_UTF_8),
            ContentError::CouldNotFetch => {
                Response::new(404).body_string(format!("Could not find {}", req.uri().path()))
            }
        })?;

    let converted = state
        .markdown_converter
        .convert_markdown(&contents)
        .await
        .map_err(|_| {
            Response::new(500).body_string(format!(
                "Could not convert the following markdown:\n {}",
                &contents
            ))
        })?;

    let resp = base_html(converted);

    Ok(Response::new(200)
        .body_string(resp)
        .set_mime(mime::TEXT_HTML_UTF_8))
}

/// Builds a `tide::Server` with the appropriate endpoint mappings.
pub fn build_app(
    state: State<
        impl MarkdownConverter + Send + Sync + 'static,
        impl ContentFinder + Send + Sync + 'static,
    >,
) -> Server<State<impl MarkdownConverter, impl ContentFinder>> {
    let mut app = Server::with_state(state);
    app.middleware(middleware::RequestLogger::new());
    app.at("").get(render_readme);
    app.at("/static/octicons/:file")
        .get(static_files::static_content);
    app.at("/*").get(render_markdown_path);

    app
}
