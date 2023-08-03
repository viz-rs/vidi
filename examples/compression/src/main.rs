#![deny(warnings)]

use std::net::SocketAddr;

use viz::{get, middleware::compression, Request, Result, Router, Server, ServiceMaker};

async fn index(_req: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {addr}");

    let app = Router::new()
        .route("/", get(index))
        .with(compression::Config);

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{err}");
    }

    Ok(())
}
