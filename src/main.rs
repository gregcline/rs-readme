#[macro_use]
extern crate serde_derive;

mod content_finder;
mod markdown_converter;

use tide;
use tide::{middleware, Request, Response, Server};

use std::env;
use std::path::PathBuf;

use content_finder::{ContentFinder, Finder};
use markdown_converter::{Converter, MarkdownConverter};

struct State<M, C>
where
    M: markdown_converter::MarkdownConverter,
    C: content_finder::ContentFinder,
{
    markdown_converter: M,
    content_finder: C,
}

async fn render_readme(req: Request<State<Converter, Finder>>) -> tide::Result {
    let state = req.state();

    let contents = state
        .content_finder
        .content_for("README.md")
        .map_err(|_| Response::new(404).body_string("Could not find README.md".to_string()))?;

    let resp = state
        .markdown_converter
        .convert_markdown(&contents)
        .await
        .map_err(|_err| {
            Response::new(500).body_string(format!(
                "Could not convert the following markdown:\n {}",
                &contents
            ))
        })?;

    Ok(Response::new(200).body_string(resp))
}

#[async_std::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    pretty_env_logger::init();
    let addr = format!(
        "0.0.0.0:{}",
        env::var("PORT").unwrap_or_else(|_| "4000".to_string())
    );

    let state = State {
        markdown_converter: Converter::new("https://api.github.com".to_string()),
        content_finder: Finder::new(PathBuf::from(".")),
    };

    let mut app = Server::with_state(state);
    app.middleware(middleware::RequestLogger::new());
    app.at("").get(render_readme);
    app.listen(addr).await
}
