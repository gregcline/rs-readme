use std::sync::Arc;
use structopt::StructOpt;

use rs_readme::{build_app, Args, Converter, Converters, FileFinder, OfflineConverter, State};

#[async_std::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    pretty_env_logger::init();

    let args = Args::from_args();

    let addr = format!("{}:{}", args.host, args.port);

    let converter = if args.offline {
        Converters::Offline(OfflineConverter::new())
    } else {
        Converters::Github(Converter::new(
            "https://api.github.com".to_string(),
            args.context,
        ))
    };

    let state = State::new(converter, FileFinder::new(args.folder));

    let app = build_app(Arc::new(state));

    println!("Listening on {}", addr);
    app.listen(addr).await
}
