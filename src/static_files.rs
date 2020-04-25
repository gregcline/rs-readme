use super::{
    content_finder::ContentFinder, markdown_converter::MarkdownConverter, web_server::State,
};
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
    req: Request<State<impl MarkdownConverter + Send + Sync, impl ContentFinder + Send + Sync>>,
) -> tide::Result {
    match req.param::<String>("file") {
        Ok(path) if path.starts_with("octicons.css") => Ok(Response::new(StatusCode::Ok)
            .body_string(OCTICON_CSS.to_string())
            .set_mime(mime::TEXT_CSS_UTF_8)),
        Ok(path) if path.starts_with("octicons.eot") => {
            Ok(Response::new(StatusCode::Ok).body(OCTICON_EOT).set_mime(
                "application/vnd.ms-fontobject"
                    .parse()
                    .unwrap_or(mime::FONT_WOFF),
            ))
        }
        Ok(path) if path.starts_with("octicons.svg") => Ok(Response::new(StatusCode::Ok)
            .body_string(OCTICON_SVG.to_string())
            .set_mime(mime::IMAGE_SVG)),
        Ok(path) if path.starts_with("octicons.ttf") => Ok(Response::new(StatusCode::Ok)
            .body(OCTICON_TTF)
            .set_mime("font/ttf".parse().unwrap())),
        Ok(path) if path.starts_with("octicons.woff2") => Ok(Response::new(StatusCode::Ok)
            .body(OCTICON_WOFF2)
            .set_mime(mime::FONT_WOFF2)),
        Ok(path) if path.starts_with("octicons.woff") => Ok(Response::new(StatusCode::Ok)
            .body(OCTICON_WOFF)
            .set_mime(mime::FONT_WOFF)),
        _ => Ok(Response::new(StatusCode::NotFound)
            .body_string("This file does not exist".to_string())
            .set_mime(mime::TEXT_HTML)),
    }
}

/// The endpoint to return our styles
pub async fn style(
    _req: Request<State<impl MarkdownConverter + Send + Sync, impl ContentFinder + Send + Sync>>,
) -> tide::Result {
    Ok(Response::new(StatusCode::Ok)
        .body_string(STYLE_CSS.to_string())
        .set_mime(mime::TEXT_CSS_UTF_8))
}
