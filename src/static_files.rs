use super::{ContentFinder, MarkdownConverter, State};
use http_types::mime;
use std::sync::Arc;
use tide::{http::StatusCode, Request, Response};

// This will bundle the necessary files in the final binary so we don't have to worry about
// portability.
const OCTICON_CSS: &str = include_str!("../static/octicons/octicons.css");
const OCTICON_EOT: &[u8] = include_bytes!("../static/octicons/octicons.eot");
const OCTICON_SVG: &str = include_str!("../static/octicons/octicons.svg");
const OCTICON_TTF: &[u8] = include_bytes!("../static/octicons/octicons.ttf");
const OCTICON_WOFF: &[u8] = include_bytes!("../static/octicons/octicons.woff");
const OCTICON_WOFF2: &[u8] = include_bytes!("../static/octicons/octicons.woff2");

const STYLE_CSS: &str = include_str!("../static/style.css");

/// The endpoint to return files related to octicons
pub async fn octicons(
    req: Request<
        Arc<State<impl MarkdownConverter + Send + Sync, impl ContentFinder + Send + Sync>>,
    >,
) -> tide::Result {
    match req.param("file") {
        Ok(path) if path.starts_with("octicons.css") => Ok(Response::builder(StatusCode::Ok)
            .body(OCTICON_CSS.to_string())
            .content_type(mime::CSS)
            .build()),
        Ok(path) if path.starts_with("octicons.eot") => Ok(Response::builder(StatusCode::Ok)
            .body(OCTICON_EOT)
            .content_type("application/vnd.ms-fontobject".parse().unwrap_or(mime::ANY))
            .build()),
        Ok(path) if path.starts_with("octicons.svg") => Ok(Response::builder(StatusCode::Ok)
            .body(OCTICON_SVG.to_string())
            .content_type(mime::SVG)
            .build()),
        Ok(path) if path.starts_with("octicons.ttf") => Ok(Response::builder(StatusCode::Ok)
            .body(OCTICON_TTF)
            .content_type("font/ttf".parse().unwrap_or(mime::ANY))
            .build()),
        Ok(path) if path.starts_with("octicons.woff2") => Ok(Response::builder(StatusCode::Ok)
            .body(OCTICON_WOFF2)
            .content_type("font/woff2".parse().unwrap_or(mime::ANY))
            .build()),
        Ok(path) if path.starts_with("octicons.woff") => Ok(Response::builder(StatusCode::Ok)
            .body(OCTICON_WOFF)
            .content_type("font/woff".parse().unwrap_or(mime::ANY))
            .build()),
        _ => Ok(Response::builder(StatusCode::NotFound)
            .body("This file does not exist".to_string())
            .content_type(mime::HTML)
            .build()),
    }
}

/// The endpoint to return our styles
pub async fn style(
    _req: Request<
        Arc<State<impl MarkdownConverter + Send + Sync, impl ContentFinder + Send + Sync>>,
    >,
) -> tide::Result {
    Ok(Response::builder(StatusCode::Ok)
        .body(STYLE_CSS.to_string())
        .content_type(mime::CSS)
        .build())
}
