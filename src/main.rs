#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate horrorshow;

mod content_finder;
mod markdown_converter;

use tide;
use tide::{middleware, Request, Response, Server};
use horrorshow::prelude::*;
use horrorshow::helper::doctype;

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

fn wrap_converted(converted: String) -> String {
    format!("{}", html! {
        : doctype::HTML;
        html {
            head {
                title : "readme-rs";
            }
            body {
                : Raw(&converted);
            }
        }
    })
}

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

    let resp = wrap_converted(converted);

    Ok(Response::new(200).body_string(resp).set_mime(mime::TEXT_HTML))
}

fn build_app(
    state: State<
        impl MarkdownConverter + Send + Sync + 'static,
        impl ContentFinder + Send + Sync + 'static,
    >,
) -> Server<State<impl MarkdownConverter, impl ContentFinder>> {
    let mut app = Server::with_state(state);
    app.middleware(middleware::RequestLogger::new());
    app.at("").get(render_readme);

    app
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

    let app = build_app(state);

    app.listen(addr).await
}

#[cfg(test)]
mod test {
    use super::*;
    use async_trait::async_trait;
    use content_finder::ContentError;
    use http_service::Body;
    use http_service_mock::make_server;
    use markdown_converter::MarkdownError;
    use async_std::io::ReadExt;

    struct MockConverter;

    #[async_trait]
    impl MarkdownConverter for MockConverter {
        async fn convert_markdown(&self, _md: &str) -> Result<String, MarkdownError> {
            Ok("<h1>A Readme</h1>".to_string())
        }
    }

    struct MockFinder;

    impl ContentFinder for MockFinder {
        fn content_for(&self, _resource: &str) -> Result<String, ContentError> {
            Ok("# A Readme".to_string())
        }
    }

    #[async_std::test]
    async fn index_wraps_in_html() {
        let state = State {
            markdown_converter: MockConverter,
            content_finder: MockFinder,
        };
        let app = build_app(state);
        let mut server = make_server(app.into_http_service()).unwrap();

        let req = http::Request::get("/").body(Body::empty()).unwrap();
        let res = server.simulate(req).unwrap();
        let status = res.status();
        let mut body = String::with_capacity(17);
        res.into_body().read_to_string(&mut body).await.unwrap();

        assert_eq!(status.as_u16(), 200);
        let expected_body = "\
<!DOCTYPE html>\
<html>\
  <head>\
    <title>readme-rs</title>\
  </head>\
  <body>\
    <h1>A Readme</h1>\
  </body>\
</html>";
        assert_eq!(body, expected_body);
    }
}
