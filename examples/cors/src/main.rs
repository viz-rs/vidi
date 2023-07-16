#![deny(warnings)]
#![allow(clippy::unused_async)]

use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use viz::{
    get, middleware::cors, server::conn::http1, Io, Method, Request, Responder, Result, Router,
    Tree,
};

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
    println!("listening on {addr}");

    let custom_cors = cors::Config::new()
        .allow_methods([Method::GET, Method::POST])
        .credentials(true);

    let app = Router::new()
        .route("/", get(index).options(options))
        // .with(cors::Config::default()); // Default CORS config
        .with(custom_cors); // Our custom CORS config
    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(Io::new(stream), Responder::new(tree, Some(addr)))
                .await
            {
                eprintln!("Error while serving HTTP connection: {err}");
            }
        });
    }
}
