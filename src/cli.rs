use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "rs-readme", about = "A simple web server for previewing .md files")]
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
}
