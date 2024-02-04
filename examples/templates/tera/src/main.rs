#![deny(warnings)]

use std::net::SocketAddr;

use once_cell::sync::Lazy;
use serde::Serialize;
use tera::{Context, Tera};
use tokio::net::TcpListener;
use viz::{serve, Error, Request, Response, ResponseExt, Result, Router};

static TPLS: Lazy<Tera> =
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
    let body = TPLS.render("index.html", &ctx).map_err(Error::boxed)?;

    Ok(Response::html(body))
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
