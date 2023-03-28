#![deny(warnings)]
#![allow(clippy::unused_async)]

use std::{net::SocketAddr, sync::Arc};

use once_cell::sync::Lazy;
use serde::Serialize;
use tera::{Context, Tera};
use tokio::net::TcpListener;
use viz::{
    server::conn::http1, BytesMut, Error, Request, Responder, Response, ResponseExt, Result,
    Router, Tree,
};

static TERA: Lazy<Tera> =
    Lazy::new(|| Tera::new("examples/templates/tera/templates/**/*").unwrap());

#[derive(Serialize)]
struct User<'a> {
    url: &'a str,
    username: &'a str,
}

async fn index(_: Request) -> Result<Response> {
    let mut ctx = Context::new();
    ctx.insert("title", "Viz.rs");
    ctx.insert(
        "users",
        &vec![
            User {
                url: "https://github.com/rust-lang",
                username: "rust-lang",
            },
            User {
                url: "https://github.com/viz-rs",
                username: "viz-rs",
            },
        ],
    );
    let mut buf = BytesMut::with_capacity(512);
    buf.extend(
        TERA.render("index.html", &ctx)
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
                .serve_connection(stream, Responder::new(tree, Some(addr)))
                .await
            {
                eprintln!("Error while serving HTTP connection: {err}");
            }
        });
    }
}
