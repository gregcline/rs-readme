use std::env;
use std::path::PathBuf;

use rs_readme::Converter;
use rs_readme::FileFinder;
use rs_readme::{build_app, State};

#[async_std::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    pretty_env_logger::init();
    let addr = format!(
        "127.0.0.1:{}",
        env::var("PORT").unwrap_or_else(|_| "4000".to_string())
    );

    let state = State::new(
        Converter::new("https://api.github.com".to_string()),
        FileFinder::new(PathBuf::from(".")),
    );

    let app = build_app(state);

    println!(
        "Listening on {}\nYou can change the port with the PORT env var",
        addr
    );
    app.listen(addr).await
}
