#![deny(warnings)]

use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use viz::{handlers::embed, serve, Result, Router, StatusCode, Tree};

#[derive(rust_embed::RustEmbed)]
#[folder = "public"]
struct Asset;

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(feature = "tracing")]
    tracing_subscriber::fmt::init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let app = Router::new()
        .get("/", embed::File::<Asset>::new("index.html"))
        .get("/static/*", embed::Dir::<Asset>::default())
        .any("/*", |_| async { Ok(StatusCode::NOT_FOUND) });
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
