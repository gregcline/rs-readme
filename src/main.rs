#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate async_std;

use surf;
use tide::{Request, Response, Result, ResultExt, Server,};

use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct MarkdownRequest {
    text: String,
    mode: String,
    context: String,
}

async fn find_readme(_req: Request<()>) -> Result {
    let path = Path::new("README.md");

    let mut file = File::open(path).unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).server_err()?;

    let client = surf::Client::new();

    let resp = client.post("https://api.github.com/markdown")
        .body_json(&MarkdownRequest {
            text: contents,
            mode: "markdown".to_string(),
            context: "".to_string(),
        })
        .server_err()?
        .recv_string()
        .await
        .map_err(|_| Response::new(500))?;

    Ok(Response::new(200).body_string(resp))
}

#[async_std::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    let addr = format!("0.0.0.0:{}", env::var("PORT").unwrap_or_else(|_| "4000".to_string()));

    println!("Listening on http://{}", addr).await;

    let mut app = Server::new();
    app.at("").get(find_readme);
    app.listen(addr).await
}
