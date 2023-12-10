#![deny(warnings)]
#![allow(clippy::unused_async)]

use std::{net::SocketAddr, sync::Arc};

use minijinja::{context, path_loader, Environment};
use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::net::TcpListener;
use viz::{serve, BytesMut, Error, Request, Response, ResponseExt, Result, Router, Tree};

static MINIJINJA: Lazy<Environment> = Lazy::new(|| {
    let mut env = Environment::new();
    env.set_loader(path_loader("examples/templates/minijinja/templates"));
    env
});

#[derive(Serialize)]
struct User<'a> {
    url: &'a str,
    username: &'a str,
}

async fn index(_: Request) -> Result<Response> {
    let mut buf = BytesMut::with_capacity(512);
    buf.extend(
        MINIJINJA
            .get_template("index.html")
            .map_err(Error::normal)?
            .render(context! {
                title => "Viz.rs",
                users => &vec![
                    User {
                        url: "https://github.com/rust-lang",
                        username: "rust-lang",
                    },
                    User {
                        url: "https://github.com/viz-rs",
                        username: "viz-rs",
                    },
                ],
            })
            .map_err(Error::normal)?
            .as_bytes(),
    );

    Ok(Response::html(buf.freeze()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let app = Router::new().get("/", index);
    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(async move {
            if let Err(err) = serve(stream, tree, Some(addr)).await {
                eprintln!("Error while serving HTTP connection: {err}");
            }
        });
    }
}
