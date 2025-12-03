#![allow(clippy::unused_async)]

use std::{env, net::SocketAddr, path::PathBuf, sync::LazyLock};

use minijinja::{Environment, context, path_loader};
use serde::Serialize;
use tokio::net::TcpListener;
use vidi::{Error, Request, Response, ResponseExt, Result, Router, serve};

static TPLS: LazyLock<Environment> = LazyLock::new(|| {
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
    let body = TPLS
        .get_template("index.html")
        .map_err(Error::boxed)?
        .render(context! {
            title => "Vidi",
            users => &vec![
                User {
                    url: "https://github.com/rust-lang",
                    username: "rust-lang",
                },
                User {
                    url: "https://github.com/viz-rs/vidi",
                    username: "vidi",
                },
            ],
        })
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
