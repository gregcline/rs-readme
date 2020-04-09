use structopt::StructOpt;

use rs_readme::{build_app, State, Converter, FileFinder, Args};

#[async_std::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    pretty_env_logger::init();

    let args = Args::from_args();

    let addr = format!(
        "{}:{}",
        args.host,
        args.port
    );

    let state = State::new(
        Converter::new("https://api.github.com".to_string()),
        FileFinder::new(args.folder),
    );

    let app = build_app(state);

    println!(
        "Listening on {}",
        addr
    );
    app.listen(addr).await
}
