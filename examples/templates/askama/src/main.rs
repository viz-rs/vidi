use std::net::SocketAddr;

use askama::Template;
use tokio::net::TcpListener;
use vidi::{Error, Request, Response, ResponseExt, Result, Router, serve};

#[derive(Template)]
#[template(path = "hello.html")]
struct HelloTemplate<'a> {
    name: &'a str,
}

async fn index(_: Request) -> Result<Response> {
    let body = HelloTemplate { name: "world" }
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
