// Create mock
use async_std::io::prelude::*;
use async_trait::async_trait;
use http_types::mime;
use pretty_assertions::assert_eq;
use rs_readme::*;
use rs_readme::{ContentError, ContentFinder, MarkdownConverter};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use tide::http::{Method, Request, Url, Response};

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
    fn content_for(&self, _resource: &str) -> Result<String, ContentError> {
        Ok("# A Readme".to_string())
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
    fn content_for(&self, resource: &str) -> Result<String, ContentError> {
        self.seen
            .lock()
            .expect("Could not lock mutex in content_for")
            .insert(resource.to_string());

        Ok(format!("content for: {}", resource).to_string())
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
    let app = build_app(Arc::new(state));

    // Request
    let req = Request::new(Method::Get, Url::parse("http://localhost/").unwrap());
    let mut res: Response = app.respond(req).await.unwrap();

    // Assert
    let status = res.status();
    assert_eq!(status, 200);

    let mime = res
        .content_type()
        .expect("Couldn't get the content-type header");

    assert_eq!(mime, mime::HTML);

    let body = res.body_string().await.unwrap();
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
  </body>\
</html>";
    assert_eq!(body, expected_body);
}

#[async_std::test]
async fn non_index_wraps_in_html() {
    // Setup
    let state = State::new(MockConverter, MockFinder);
    let app = build_app(Arc::new(state));

    // Request
    let req = Request::new(Method::Get, Url::parse("http://localhost/foo.md").unwrap());
    let mut res: Response = app.respond(req).await.unwrap();

    // Assert
    let status = res.status();
    assert_eq!(status, 200);

    let mime = res
        .content_type()
        .expect("Couldn't get the content-type header");

    assert_eq!(mime, mime::HTML);

    let body = res.body_string().await.unwrap();
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
    let app = build_app(Arc::new(state));

    // Request
    let req = Request::new(
        Method::Get,
        Url::parse("http://localhost/test_dir/a.md").unwrap(),
    );
    let res: Response = app.respond(req).await.unwrap();

    // Assert
    let status = res.status();
    assert_eq!(status, 200);

    let mime = res
        .content_type()
        .expect("Couldn't get the content-type header");
    assert_eq!(mime, mime::HTML);

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
        fn content_for(&self, _resource: &str) -> Result<String, ContentError> {
            Err(ContentError::NotMarkdown)
        }
    }

    // Setup
    let state = State::new(MockConverter, MockFinderError);
    let app = build_app(Arc::new(state));

    // Request
    let req = Request::new(Method::Get, Url::parse("http://localhost/foo.txt").unwrap());
    let mut res: Response = app.respond(req).await.unwrap();

    // Assert
    let status = res.status();
    assert_eq!(status, 400);

    let mime = res
        .content_type()
        .expect("Couldn't get the content-type header");
    assert_eq!(mime, mime::HTML);

    let body = res.body_string().await.unwrap();
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
async fn returns_404_for_missing_readme() {
    // Create mock
    struct MockFinderError;

    impl ContentFinder for MockFinderError {
        fn content_for(&self, resource: &str) -> Result<String, ContentError> {
            Err(ContentError::CouldNotFetch(resource.to_string()))
        }
    }

    // Setup
    let state = State::new(MockConverter, MockFinderError);
    let app = build_app(Arc::new(state));

    // Request
    let req = Request::new(Method::Get, Url::parse("http://localhost/").unwrap());
    let mut res: Response = app.respond(req).await.unwrap();

    // Assert
    let status = res.status();
    assert_eq!(status, 404);

    let mime = res
        .content_type()
        .expect("Couldn't get the content-type header");
    assert_eq!(mime, mime::HTML);

    let body = res.body_string().await.unwrap();
    let expected_body = "Could not find README.md";
    assert_eq!(body, expected_body);
}

#[async_std::test]
async fn returns_404_for_missing_file() {
    // Create mock
    struct MockFinderError;

    impl ContentFinder for MockFinderError {
        fn content_for(&self, resource: &str) -> Result<String, ContentError> {
            Err(ContentError::CouldNotFetch(resource.to_string()))
        }
    }

    // Setup
    let state = State::new(MockConverter, MockFinderError);
    let app = build_app(Arc::new(state));

    // Request
    let req = Request::new(Method::Get, Url::parse("http://localhost/foo.md").unwrap());
    let mut res: Response = app.respond(req).await.unwrap();

    // Assert
    let status = res.status();
    assert_eq!(status, 404);

    let mime = res
        .content_type()
        .expect("Couldn't get the content-type header");
    assert_eq!(mime, mime::HTML);

    let body = res.body_string().await.unwrap();
    let expected_body = "Could not find foo.md";
    assert_eq!(body, expected_body);
}

#[async_std::test]
async fn static_content_returns_appropriate_files() {
    // Setup
    let state = State::new(MockConverter, MockFinder);
    let app = build_app(Arc::new(state));

    // Expected results
    // (path, status, mime, body)
    let expected = vec![
        (
            "/static/octicons/octicons.css",
            200 as u16,
            mime::CSS,
            {
                let mut vec = Vec::new();
                vec.extend_from_slice(include_bytes!("../static/octicons/octicons.css"));
                vec
            },
        ),
        (
            "/static/octicons/octicons.eot",
            200,
            "application/vnd.ms-fontobject".parse().unwrap(),
            {
                let mut vec = Vec::new();
                vec.extend_from_slice(include_bytes!("../static/octicons/octicons.eot"));
                vec
            },
        ),
        ("/static/octicons/octicons.svg", 200, mime::SVG, {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.svg"));
            vec
        }),
        ("/static/octicons/octicons.ttf", 200, "font/ttf".parse().unwrap(), {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.ttf"));
            vec
        }),
        ("/static/octicons/octicons.woff", 200, "font/woff".parse().unwrap(), {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.woff"));
            vec
        }),
        ("/static/octicons/octicons.woff2", 200, "font/woff2".parse().unwrap(), {
            let mut vec = Vec::new();
            vec.extend_from_slice(include_bytes!("../static/octicons/octicons.woff2"));
            vec
        }),
    ];

    for (path, status, mime, body) in expected.iter() {
        // Make request
        let req = Request::new(
            Method::Get,
            Url::parse(&format!("http://localhost{}", *path)).unwrap(),
        );
        let mut res: Response = app.respond(req).await.unwrap();

        // Assert
        let res_status = res.status();
        assert_eq!(&res_status, status, "path: {}", path);

        let res_mime = res
            .content_type()
            .expect(&format!("Couldn't get the content-type header, path: {}", path));
        assert_eq!(res_mime, *mime, "path: {}", path);

        let mut res_body = Vec::with_capacity(1);
        res.take_body().read_to_end(&mut res_body).await.unwrap();

        assert_eq!(&res_body, body, "path: {}", path);
    }
}

#[async_std::test]
async fn styles_returns_right_css() {
    // Setup
    let state = State::new(MockConverter, MockFinder);
    let app = build_app(Arc::new(state));

    // Make request
    let req = Request::new(
        Method::Get,
        Url::parse("http://localhost/static/style.css").unwrap(),
    );
    let mut res: Response = app.respond(req).await.unwrap();

    // Assert
    let res_status = res.status();
    assert_eq!(res_status, 200);

    let mime = res
        .content_type()
        .expect("Couldn't get the content-type header");
    assert_eq!(mime, mime::CSS);

    let body = res.body_string().await.unwrap();

    assert_eq!(&body, include_str!("../static/style.css"));
}
