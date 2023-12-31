#![deny(warnings)]

use std::net::SocketAddr;
use tokio::net::TcpListener;
use viz::{get, middleware::cors, serve, Method, Request, Result, Router};

async fn index(_req: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

async fn options(_req: Request) -> Result<&'static str> {
    Ok("No Content!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let custom_cors = cors::Config::new()
        .allow_methods([Method::GET, Method::POST])
        .credentials(true);

    let app = Router::new()
        .route("/", get(index).options(options))
        // .with(cors::Config::default()); // Default CORS config
        .with(custom_cors); // Our custom CORS config

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
