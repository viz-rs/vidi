#![allow(clippy::unused_async)]

use std::net::SocketAddr;

use rinja::Template;
use serde::Serialize;
use tokio::net::TcpListener;
use viz::{serve, Error, Request, Response, ResponseExt, Result, Router};

#[allow(dead_code)]
#[derive(Template)]
#[template(path = "index.html")]
struct Tmpl<'a> {
    title: &'a str,
    users: Vec<User<'a>>,
}

#[derive(Serialize)]
struct User<'a> {
    url: &'a str,
    username: &'a str,
}

async fn index(_: Request) -> Result<Response> {
    let body = Tmpl {
        title: "Viz.rs",
        users: vec![
            User {
                url: "https://github.com/rust-lang",
                username: "rust-lang",
            },
            User {
                url: "https://github.com/viz-rs",
                username: "viz-rs",
            },
        ],
    }
    .render()
    .map_err(Error::boxed)?;

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
