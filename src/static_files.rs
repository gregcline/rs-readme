use super::{
    content_finder::ContentFinder, markdown_converter::MarkdownConverter, web_server::State,
};
use sha1::Sha1;
use tide::{Request, Response};

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
    let if_none_match = req.header("If-None-Match");

    match req.param::<String>("file") {
        Ok(path) if path.starts_with("octicons.css") => {
            let digest = format!("\"{}\"", Sha1::from(OCTICON_CSS).hexdigest());
            if if_none_match == Some(&digest) {
                return Ok(Response::new(304));
            }
            Ok(Response::new(200)
                .set_header("ETag", &digest)
                .body_string(OCTICON_CSS.to_string())
                .set_mime(mime::TEXT_CSS_UTF_8))
        }
        Ok(path) if path.starts_with("octicons.eot") => {
            let digest = format!("\"{}\"", Sha1::from(OCTICON_EOT).hexdigest());
            if if_none_match == Some(&digest) {
                return Ok(Response::new(304));
            }
            Ok(Response::new(200)
                .set_header("ETag", &digest)
                .body(OCTICON_EOT)
                .set_mime(
                    "application/vnd.ms-fontobject"
                        .parse()
                        .unwrap_or(mime::FONT_WOFF),
                ))
        }
        Ok(path) if path.starts_with("octicons.svg") => {
            let digest = format!("\"{}\"", Sha1::from(OCTICON_SVG).hexdigest());
            if if_none_match == Some(&digest) {
                return Ok(Response::new(304));
            }
            Ok(Response::new(200)
                .set_header("ETag", &digest)
                .body_string(OCTICON_SVG.to_string())
                .set_mime(mime::IMAGE_SVG))
        }
        Ok(path) if path.starts_with("octicons.ttf") => {
            let digest = format!("\"{}\"", Sha1::from(OCTICON_TTF).hexdigest());
            if if_none_match == Some(&digest) {
                return Ok(Response::new(304));
            }
            Ok(Response::new(200)
                .set_header("ETag", &digest)
                .body(OCTICON_TTF)
                .set_mime("font/ttf".parse().unwrap()))
        }
        Ok(path) if path.starts_with("octicons.woff2") => {
            let digest = format!("\"{}\"", Sha1::from(OCTICON_WOFF2).hexdigest());
            if if_none_match == Some(&digest) {
                return Ok(Response::new(304));
            }
            Ok(Response::new(200)
                .set_header("ETag", &digest)
                .body(OCTICON_WOFF2)
                .set_mime(mime::FONT_WOFF2))
        }
        Ok(path) if path.starts_with("octicons.woff") => {
            let digest = format!("\"{}\"", Sha1::from(OCTICON_WOFF).hexdigest());
            if if_none_match == Some(&digest) {
                return Ok(Response::new(304));
            }
            Ok(Response::new(200)
                .set_header("ETag", &digest)
                .body(OCTICON_WOFF)
                .set_mime(mime::FONT_WOFF))
        }
        _ => Ok(Response::new(404)
            .body_string("This file does not exist".to_string())
            .set_mime(mime::TEXT_HTML)),
    }
}

/// The endpoint to return our styles
pub async fn style(
    req: Request<State<impl MarkdownConverter + Send + Sync, impl ContentFinder + Send + Sync>>,
) -> tide::Result {
    let digest = format!("\"{}\"", Sha1::from(STYLE_CSS).hexdigest());

    if req.header("If-None-Match") == Some(&digest) {
        return Ok(Response::new(304));
    }

    Ok(Response::new(200)
        .set_header("ETag", &digest)
        .body_string(STYLE_CSS.to_string())
        .set_mime(mime::TEXT_CSS_UTF_8))
}
