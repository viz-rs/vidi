#![deny(warnings)]

use std::net::SocketAddr;
use tokio::net::TcpListener;

use viz::{get, middleware::compression, serve, Request, Result, Router};

async fn index(_req: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let app = Router::new()
        .route("/", get(index))
        .with(compression::Config);

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
