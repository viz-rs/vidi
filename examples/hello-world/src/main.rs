#![deny(warnings)]
#![allow(clippy::unused_async)]

use std::{net::SocketAddr, str::FromStr, sync::Arc};
use tokio::net::TcpListener;
use viz::{serve, Request, Result, Router, Tree};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
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

    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        tokio::task::spawn(serve(stream, tree.clone(), Some(addr)));
    }
}
