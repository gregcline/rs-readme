// Create mock
use async_std::io::ReadExt;
use async_trait::async_trait;
use http_service::Body;
use http_service_mock::make_server;
use pretty_assertions::assert_eq;
use rs_readme::*;
use rs_readme::{ContentError, ContentFinder, MarkdownConverter};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

/// A mock [`MarkdownConverter`] that returns:
/// `<h1>A Readme</h1>`
struct MockConverter;

#[async_trait]
impl MarkdownConverter for MockConverter {
    async fn convert_markdown(&self, _md: &str) -> Result<String, MarkdownError> {
        Ok("<h1>A Readme</h1>".to_string())
    }
}

/// A mock [`ContentFinder`] the returns:
/// `# A Readme`
struct MockFinder;

impl ContentFinder for MockFinder {
    fn content_for(&self, _resource: &str) -> Result<(String, String), ContentError> {
        Ok(("# A Readme".to_string(), "foo".to_string()))
    }
}

/// A mock [`ContentFinder`] and [`MarkdownConverter`] that keeps track of arguments
///
/// Intended to be used to verify that an endpoint is calling its dependencies in
/// the expected way. It takes an `Arc<Mutex<HashSet<String>>>` so you can query
/// the `HashSet` later to verify what was placed in it.
///
/// The `Arc` and `Mutex` are necessary for working across threads/async runtimes.
struct MockAssertSeen {
    seen: Arc<Mutex<HashSet<String>>>,
}

impl MockAssertSeen {
    fn new(seen: Arc<Mutex<HashSet<String>>>) -> MockAssertSeen {
        MockAssertSeen { seen }
    }
}

impl ContentFinder for MockAssertSeen {
    fn content_for(&self, resource: &str) -> Result<(String, String), ContentError> {
        self.seen
            .lock()
            .expect("Could not lock mutex in content_for")
            .insert(resource.to_string());

        Ok((
            format!("content for: {}", resource).to_string(),
            format!("content for: {}", resource).to_string(),
        ))
    }
}

#[async_trait]
impl MarkdownConverter for MockAssertSeen {
    async fn convert_markdown(&self, md: &str) -> Result<String, MarkdownError> {
        self.seen
            .lock()
            .expect("Could not lock mutex in convert_markdown")
            .insert(md.to_string());

        Ok(md.to_string())
    }
}

#[async_std::test]
async fn index_wraps_in_html() {
    // Setup
    let state = State::new(MockConverter, MockFinder);
    let app = build_app(state);
    let mut server = make_server(app.into_http_service()).unwrap();

    // Request
    let req = http::Request::get("/").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();

    // Assert
    let status = res.status();
    assert_eq!(status.as_u16(), 200);

    // End the borrow of res so we can consume it for the body
    {
        let mime = res
            .headers()
            .get("content-type")
            .expect("Could not get content-type");
        assert_eq!(mime, "text/html; charset=utf-8");
    }

    let mut body = String::with_capacity(1);
    res.into_body().read_to_string(&mut body).await.unwrap();
    let expected_body = "\
<!DOCTYPE html>\
<html>\
  <head>\
  <link rel=\"stylesheet\" href=\"/static/octicons/octicons.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/frameworks-146fab5ea30e8afac08dd11013bb4ee0.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/site-897ad5fdbe32a5cd67af5d1bdc68a292.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/github-c21b6bf71617eeeb67a56b0d48b5bb5c.css\">\
  <link rel=\"stylesheet\" href=\"/static/style.css\">\
    <title>README.md</title>\
  </head>\
  <body>\
    <div class=\"page\">\
      <div id=\"preview-page\" class=\"preview-page\">\
        <div role=\"main\" class=\"main-content\">\
          <div class=\"container new-discussion-timeline experiment-repo-nav\">\
            <div class=\"repository-content\">\
              <div id=\"readme\" class=\"readme boxed-group clearfix announce instapaper_body md\">\
                <h3>\
                  <span class=\"octicon octicon-book\"></span> \
                  README.md\
                </h3>\
                <article class=\"markdown-body entry-content\" itemprop=\"text\">\
                  <h1>A Readme</h1>\
                </article>\
              </div>\
            </div>\
          </div>\
        </div>\
      </div>\
      <div>&nbsp;</div>\
    </div>\
    <script>\
        setInterval(function() { location.reload(); }, 3000);\
    </script>\
  </body>\
</html>";
    assert_eq!(body, expected_body);
}

#[async_std::test]
async fn non_index_wraps_in_html() {
    // Setup
    let state = State::new(MockConverter, MockFinder);
    let app = build_app(state);
    let mut server = make_server(app.into_http_service()).unwrap();

    // Request
    let req = http::Request::get("/foo.md").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();

    // Assert
    let status = res.status();
    assert_eq!(status.as_u16(), 200);

    // End the borrow of res so we can consume it for the body
    {
        let mime = res
            .headers()
            .get("content-type")
            .expect("Could not get content-type");
        assert_eq!(mime, "text/html; charset=utf-8");
    }

    let mut body = String::with_capacity(1);
    res.into_body().read_to_string(&mut body).await.unwrap();
    let expected_body = "\
<!DOCTYPE html>\
<html>\
  <head>\
  <link rel=\"stylesheet\" href=\"/static/octicons/octicons.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/frameworks-146fab5ea30e8afac08dd11013bb4ee0.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/site-897ad5fdbe32a5cd67af5d1bdc68a292.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/github-c21b6bf71617eeeb67a56b0d48b5bb5c.css\">\
  <link rel=\"stylesheet\" href=\"/static/style.css\">\
    <title>foo.md</title>\
  </head>\
  <body>\
    <div class=\"page\">\
      <div id=\"preview-page\" class=\"preview-page\">\
        <div role=\"main\" class=\"main-content\">\
          <div class=\"container new-discussion-timeline experiment-repo-nav\">\
            <div class=\"repository-content\">\
              <div id=\"readme\" class=\"readme boxed-group clearfix announce instapaper_body md\">\
                <h3>\
                  <span class=\"octicon octicon-book\"></span> \
                  foo.md\
                </h3>\
                <article class=\"markdown-body entry-content\" itemprop=\"text\">\
                  <h1>A Readme</h1>\
                </article>\
              </div>\
            </div>\
          </div>\
        </div>\
      </div>\
      <div>&nbsp;</div>\
    </div>\
    <script>\
        setInterval(function() { location.reload(); }, 3000);\
    </script>\
  </body>\
</html>";
    assert_eq!(body, expected_body);
}

#[async_std::test]
async fn calls_content_finder_with_file_path() {
    // Setup
    let converter = Arc::new(Mutex::new(HashSet::new()));
    let finder = Arc::new(Mutex::new(HashSet::new()));
    let state = State::new(
        MockAssertSeen::new(converter.clone()),
        MockAssertSeen::new(finder.clone()),
    );
    let app = build_app(state);
    let mut server = make_server(app.into_http_service()).unwrap();

    // Request
    let req = http::Request::get("/test_dir/a.md")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();

    // Assert
    let status = res.status();
    assert_eq!(status.as_u16(), 200);

    // End the borrow of res so we can consume it for the body
    {
        let mime = res
            .headers()
            .get("content-type")
            .expect("Could not get content-type");
        assert_eq!(mime, "text/html; charset=utf-8");
    }

    assert!(finder
        .lock()
        .expect("Could not lock in finder assert")
        .contains("./test_dir/a.md"));
    assert!(converter
        .lock()
        .expect("Could not lock in converter assert")
        .contains("content for: ./test_dir/a.md"));
}

#[async_std::test]
async fn returns_400_for_non_md_file() {
    // Create mock
    struct MockFinderError;

    impl ContentFinder for MockFinderError {
        fn content_for(&self, _resource: &str) -> Result<(String, String), ContentError> {
            Err(ContentError::NotMarkdown)
        }
    }

    // Setup
    let state = State::new(MockConverter, MockFinderError);
    let app = build_app(state);
    let mut server = make_server(app.into_http_service()).unwrap();

    // Request
    let req = http::Request::get("/foo.txt").body(Body::empty()).unwrap();
    let res = server.simulate(req).unwrap();

    // Assert
    let status = res.status();
    assert_eq!(status.as_u16(), 400);

    // End the borrow of res so we can consume it for the body
    {
        let mime = res
            .headers()
            .get("content-type")
            .expect("Could not get content-type");
        assert_eq!(mime, "text/html; charset=utf-8");
    }

    let mut body = String::with_capacity(1);
    res.into_body().read_to_string(&mut body).await.unwrap();
    let expected_body = "\
<!DOCTYPE html>\
<html>\
  <head>\
  <link rel=\"stylesheet\" href=\"/static/octicons/octicons.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/frameworks-146fab5ea30e8afac08dd11013bb4ee0.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/site-897ad5fdbe32a5cd67af5d1bdc68a292.css\">\
  <link rel=\"stylesheet\" href=\"https://github.githubassets.com/assets/github-c21b6bf71617eeeb67a56b0d48b5bb5c.css\">\
  <link rel=\"stylesheet\" href=\"/static/style.css\">\
    <title>rs-readme</title>\
  </head>\
  <body>\
    <h1>Not a Markdown File</h1>\
    <p><strong>/foo.txt</strong> is not a markdown file and cannot be rendered</p>\
  </body>\
</html>";
    assert_eq!(body, expected_body);
}

#[async_std::test]
async fn etag_has_the_content_digest() {
    // Setup
    let state = State::new(MockConverter, MockFinder);
    let app = build_app(state);
    let mut server = make_server(app.into_http_service()).unwrap();

    // (route, etag, status)
    let expected = vec![("/", "\"foo\"", 200), ("/foo.md", "\"foo\"", 200)];

    // Request
    for (route, etag, status) in expected {
        let req = http::Request::get(route).body(Body::empty()).unwrap();
        let res = server.simulate(req).unwrap();

        // Assert
        let res_status = res.status();
        assert_eq!(res_status.as_u16(), status, "Route: {}", route);

        let res_etag = res
            .headers()
            .get("ETag")
            .expect(&format!("Could not get ETag\nRoute: {}", route));
        assert_eq!(res_etag, etag, "Route: {}", route);
    }
}

#[async_std::test]
async fn respects_if_none_match() {
    // Setup
    let state = State::new(MockConverter, MockFinder);
    let app = build_app(state);
    let mut server = make_server(app.into_http_service()).unwrap();

    // (route, etag, status)
    let expected = vec![("/", "\"foo\"", 304), ("/foo.md", "\"foo\"", 304)];

    // Request
    for (route, etag, status) in expected {
        let req = http::Request::get(route)
            .header("If-None-Match", etag)
            .body(Body::empty())
            .unwrap();
        let res = server.simulate(req).unwrap();

        // Assert
        let res_status = res.status();
        assert_eq!(res_status.as_u16(), status, "Route: {}", route);
    }
}

#[async_std::test]
async fn static_content_returns_appropriate_files() {
    // Setup
    let state = State::new(MockConverter, MockFinder);
    let app = build_app(state);
    let mut server = make_server(app.into_http_service()).unwrap();

    // Expected results
    // (path, status, mime, body)
    let expected = vec![
        (
            "/static/octicons/octicons.css",
            200,
            "text/css; charset=utf-8",
            {
                let mut vec = Vec::new();
                vec.extend_from_slice(include_bytes!("../static/octicons/octicons.css"));
                vec
            },
        ),
        (
            "/static/octicons/octicons.eot",
            200,
            "application/vnd.ms-fontobject",
            {
                let mut vec = Vec::new();
                vec.extend_from_slice(include_bytes!("../static/octicons/octicons.eot"));
                vec
            },
        ),
        ("/static/octicons/octicons.svg", 200, "image/svg+xml", {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.svg"));
            vec
        }),
        ("/static/octicons/octicons.ttf", 200, "font/ttf", {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.ttf"));
            vec
        }),
        ("/static/octicons/octicons.woff", 200, "font/woff", {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.woff"));
            vec
        }),
        ("/static/octicons/octicons.woff2", 200, "font/woff2", {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.woff2"));
            vec
        }),
    ];

    for (path, status, mime, body) in expected.iter() {
        // Make request
        let req = http::Request::get(*path).body(Body::empty()).unwrap();
        let res = server.simulate(req).unwrap();

        // Assert
        let res_status = res.status();
        assert_eq!(&res_status.as_u16(), status, "path: {}", path);

        // End the borrow of res so we can consume it for the body
        {
            let headers = res.headers();
            let res_mime = headers
                .get("content-type")
                .expect(&format!("Could not get content-type\nPath: {}", path));
            assert_eq!(res_mime, mime);

            let res_etag = headers
                .get("etag")
                .expect(&format!("Could not get etag\nPath: {}", path));
            assert_eq!(
                res_etag,
                &format!("\"{}\"", &sha1::Sha1::from(body).hexdigest())
            )
        }

        let mut res_body: Vec<u8> = Vec::with_capacity(1);
        res.into_body().read_to_end(&mut res_body).await.unwrap();

        assert_eq!(&res_body, body, "path: {}", path);
    }
}

#[async_std::test]
async fn styles_returns_right_css() {
    // Setup
    let state = State::new(MockConverter, MockFinder);
    let app = build_app(state);
    let mut server = make_server(app.into_http_service()).unwrap();
    let style_body = include_str!("../static/style.css");

    // Make request
    let req = http::Request::get("/static/style.css")
        .body(Body::empty())
        .unwrap();
    let res = server.simulate(req).unwrap();

    // Assert
    let res_status = res.status();
    assert_eq!(res_status.as_u16(), 200);

    // End the borrow of res so we can consume it for the body
    {
        let headers = res.headers();
        let res_mime = headers
            .get("content-type")
            .expect("Could not get content-type");
        assert_eq!(res_mime, "text/css; charset=utf-8");

        let res_etag = headers.get("etag").expect("Could not get etag");
        assert_eq!(
            res_etag,
            &format!("\"{}\"", &sha1::Sha1::from(style_body).hexdigest())
        )
    }

    let mut res_body = String::with_capacity(1);
    res.into_body().read_to_string(&mut res_body).await.unwrap();

    assert_eq!(&res_body, style_body);
}

#[async_std::test]
async fn static_content_respects_if_none_match() {
    // Setup
    let state = State::new(MockConverter, MockFinder);
    let app = build_app(state);
    let mut server = make_server(app.into_http_service()).unwrap();

    // Expected results
    // (path, status, mime, body)
    let expected = vec![
        ("/static/style.css", 304, {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/style.css"));
            vec
        }),
        ("/static/octicons/octicons.css", 304, {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.css"));
            vec
        }),
        ("/static/octicons/octicons.eot", 304, {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.eot"));
            vec
        }),
        ("/static/octicons/octicons.svg", 304, {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.svg"));
            vec
        }),
        ("/static/octicons/octicons.ttf", 304, {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.ttf"));
            vec
        }),
        ("/static/octicons/octicons.woff", 304, {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.woff"));
            vec
        }),
        ("/static/octicons/octicons.woff2", 304, {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.woff2"));
            vec
        }),
    ];

    for (path, status, body) in expected.iter() {
        let digest = sha1::Sha1::from(body).hexdigest();
        // Make request
        let req = http::Request::get(*path)
            .header("If-None-Match", format!("\"{}\"", digest))
            .body(Body::empty())
            .unwrap();
        let res = server.simulate(req).unwrap();

        // Assert
        let res_status = res.status();
        assert_eq!(&res_status.as_u16(), status, "path: {}", path);
    }
}
