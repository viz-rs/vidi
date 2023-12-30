#![deny(warnings)]
#![allow(clippy::unused_async)]

use std::{net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;
use viz::{Request, Result, Router, Server, Tree};

async fn index(_: Request) -> Result<String> {
    Ok(String::from("Hello, World!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from_str("[::1]:3000").unwrap();
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let mut app = Router::new().get("/", |_| async { Ok("Hello, World!") });

    for n in 0..1000 {
        app = app.get(&format!("/{}", n), index);
    }

    let tree = Tree::from(app);

    let server = Server::new(listener, tree);

    if let Err(_) = server.await {}

    Ok(())
}
