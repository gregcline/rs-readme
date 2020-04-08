use std::env;
use std::path::PathBuf;

use rs_readme::Converter;
use rs_readme::FileFinder;
use rs_readme::{build_app, State};

#[async_std::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    pretty_env_logger::init();
    let addr = format!(
        "0.0.0.0:{}",
        env::var("PORT").unwrap_or_else(|_| "4000".to_string())
    );

    let state = State::new(
        Converter::new("https://api.github.com".to_string()),
        FileFinder::new(PathBuf::from(".")),
    );

    let app = build_app(state);

    app.listen(addr).await
}
