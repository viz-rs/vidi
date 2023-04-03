#![deny(warnings)]
#![allow(clippy::unused_async)]

use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use viz::{get, server::conn::http1, tls, Request, Responder, Result, Router, Tree};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {addr}");

    let app = Router::new().route("/", get(index));
    let tree = Arc::new(Tree::from(app));

    let listener = tls::Listener::<_, tls::rustls::TlsAcceptor>::new(
        listener,
        tls::rustls::Config::new()
            .cert(include_bytes!("../../tls/cert.pem").to_vec())
            .key(include_bytes!("../../tls/key.pem").to_vec())
            .build()
            .map(Arc::new)?
            .into(),
    );

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, Responder::new(tree, Some(addr)))
                .await
            {
                eprintln!("Error while serving HTTP connection: {err}");
            }
        });
    }
}
