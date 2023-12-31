#![deny(warnings)]

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use viz::{
    get, middleware::limits, serve, types::Form, IntoHandler, Request, Response, ResponseExt,
    Result, Router,
};

#[derive(Deserialize, Serialize)]
struct Post {
    title: String,
    content: String,
}

// HTML form for creating a post
async fn new(_: Request) -> Result<Response> {
    Ok(Response::html(include_str!("../index.html")))
}

// create a post
async fn create(Form(post): Form<Post>) -> Result<Response> {
    Ok(Response::json(post)?)
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let app = Router::new()
        .route("/", get(new).post(create.into_handler()))
        // limit body size
        .with(limits::Config::default());

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
