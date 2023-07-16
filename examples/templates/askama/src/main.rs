#![deny(warnings)]
#![allow(clippy::unused_async)]

use std::{net::SocketAddr, sync::Arc};

use askama::Template;
use tokio::net::TcpListener;
use viz::{
    server::conn::http1, BytesMut, Error, Io, Request, Responder, Response, ResponseExt, Result,
    Router, Tree,
};

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

async fn index(_: Request) -> Result<Response> {
    let mut buf = BytesMut::with_capacity(512);
    buf.extend(
        HelloTemplate { name: "world" }
            .render()
            .map_err(Error::normal)?
            .as_bytes(),
    );

    Ok(Response::html(buf.freeze()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {addr}");

    let app = Router::new().get("/", index);
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
