#![deny(warnings)]

use std::net::SocketAddr;
use viz::{middleware::limits, Request, RequestExt, Result, Router, Server, ServiceMaker};

async fn echo(mut req: Request) -> Result<String> {
    Ok(format!("len: {}", req.text().await?.len()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {addr}");

    let limits = viz::types::Limits::new()
        .insert("payload", 1000 * 1024 * 1024)
        .insert("text", 1000 * 1024 * 1024)
        .insert("bytes", 1000 * 1024 * 1024)
        .insert("json", 1000 * 1024 * 1024);

    let app = Router::new()
        .post("/", echo)
        // limit body size
        .with(limits::Config::default().limits(limits));

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{err}");
    }

    Ok(())
}
