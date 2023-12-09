#![deny(warnings)]
#![allow(clippy::unused_async)]

use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use viz::{handlers::serve, serve, Request, Response, ResponseExt, Result, Router, Tree};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    // in `viz` directory
    let dir = env::current_dir().unwrap();

    let app = Router::new()
        .get("/", index)
        .get("/cargo.toml", serve::File::new(dir.join("Cargo.toml")))
        .get(
            "/examples/*",
            serve::Dir::new(dir.join("examples")).listing(),
        )
        .any("/*", |_| async { Ok(Response::text("Welcome!")) });
    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(async move {
            if let Err(err) = serve(stream, tree, Some(addr)).await {
                eprintln!("Error while serving HTTP connection: {err}");
            }
        });
    }
}
