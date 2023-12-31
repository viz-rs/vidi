#![deny(warnings)]
#![allow(clippy::unused_async)]

use std::{env, net::SocketAddr, path::PathBuf};

use minijinja::{context, path_loader, Environment};
use once_cell::sync::Lazy;
use serde::Serialize;
use tokio::net::TcpListener;
use viz::{serve, BytesMut, Error, Request, Response, ResponseExt, Result, Router};

static TPLS: Lazy<Environment> = Lazy::new(|| {
    let dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
    let mut env = Environment::new();
    env.set_loader(path_loader(dir.join("templates")));
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
        TPLS.get_template("index.html")
            .map_err(Error::boxed)?
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
            .map_err(Error::boxed)?
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

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
