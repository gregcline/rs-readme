use super::{content_finder::ContentFinder, markdown_converter::MarkdownConverter, State};
use tide::{Request, Response};

const OCTICON_CSS: &'static str = include_str!("../static/octicons/octicons.css");
const OCTICON_EOT: &[u8] = include_bytes!("../static/octicons/octicons.eot");
const OCTICON_SVG: &'static str = include_str!("../static/octicons/octicons.svg");
const OCTICON_TTF: &[u8] = include_bytes!("../static/octicons/octicons.ttf");
const OCTICON_WOFF: &[u8] = include_bytes!("../static/octicons/octicons.woff");
const OCTICON_WOFF2: &[u8] = include_bytes!("../static/octicons/octicons.woff2");

pub async fn static_content(
    req: Request<State<impl MarkdownConverter + Send + Sync, impl ContentFinder + Send + Sync>>,
) -> tide::Result {
    match req.uri().path() {
        path if path.starts_with("/static/octicons/octicons.css") => Ok(Response::new(200)
            .body_string(OCTICON_CSS.to_string())
            .set_mime(mime::TEXT_CSS_UTF_8)),
        path if path.starts_with("/static/octicons/octicons.eot") => {
            Ok(Response::new(200).body(OCTICON_EOT).set_mime(
                "application/vnd.ms-fontobject"
                    .parse()
                    .unwrap_or(mime::FONT_WOFF),
            ))
        }
        path if path.starts_with("/static/octicons/octicons.svg") => Ok(Response::new(200)
            .body_string(OCTICON_SVG.to_string())
            .set_mime(mime::IMAGE_SVG)),
        path if path.starts_with("/static/octicons/octicons.ttf") => Ok(Response::new(200)
            .body(OCTICON_TTF)
            .set_mime("font/ttf".parse().unwrap())),
        path if path.starts_with("/static/octicons/octicons.woff2") => Ok(Response::new(200)
            .body(OCTICON_WOFF2)
            .set_mime(mime::FONT_WOFF2)),
        path if path.starts_with("/static/octicons/octicons.woff") => Ok(Response::new(200)
            .body(OCTICON_WOFF)
            .set_mime(mime::FONT_WOFF)),
        _ => Ok(Response::new(404)
            .body_string("This file does not exist".to_string())
            .set_mime(mime::TEXT_HTML)),
    }
}
