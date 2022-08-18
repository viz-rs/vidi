#![deny(warnings)]

use std::env;
use std::net::SocketAddr;
use viz::{handlers::serve, Request, Response, ResponseExt, Result, Router, Server, ServiceMaker};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let dir = env::current_dir().unwrap();

    let app = Router::new()
        .get("/", index)
        .get("/cargo.toml", serve::File::new(dir.join("Cargo.toml")))
        .get("/examples/*", serve::Dir::new(dir).listing())
        .any("/*", |_| async { Ok(Response::text("Welcome!")) });

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
