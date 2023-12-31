#![deny(warnings)]

use std::net::SocketAddr;
use tokio::net::TcpListener;
use viz::{
    middleware::limits,
    serve,
    // types::{Multipart, PayloadError},
    types,
    Request,
    RequestExt,
    Result,
    Router,
};

async fn echo(mut req: Request) -> Result<String> {
    Ok(format!("len: {}", req.text().await?.len()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let limits = types::Limits::new()
        .set("bytes", 1024 * 8) // 8KB
        .set("form", 1024 * 1024) // 1MB
        .set("json", 1024 * 1024) // 1MB
        .set("payload", 1024 * 8) // 8KB
        .set("text", 1024 * 8); // 8KB

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {addr}");

    let app = Router::new()
        .post("/", echo)
        // limit body size
        .with(limits::Config::default().limits(limits));

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
