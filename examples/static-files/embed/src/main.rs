#![deny(warnings)]

use std::net::SocketAddr;
use tokio::net::TcpListener;
use viz::{handlers::embed, serve, Result, Router, StatusCode};

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

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
