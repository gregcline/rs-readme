use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "rs-readme",
    about = "A simple web server for previewing .md files"
)]
pub struct Args {
    /// The host to serve the readme files on
    #[structopt(short, long, default_value = "127.0.0.1")]
    pub host: String,

    /// The port to serve the readme files on
    #[structopt(short, long, default_value = "4000")]
    pub port: usize,

    /// The folder to use as the root when serving files
    #[structopt(short, long, default_value = ".")]
    pub folder: PathBuf,

    /// The GitHub context to render in, should be of the form: `user/repo` or `org/repo`
    #[structopt(short, long)]
    pub context: Option<String>,

    /// Whether to run in offline mode, using a built in markdown converter. May
    /// not be 100% accurate to GitHub
    #[structopt(short, long)]
    pub offline: bool,
}
