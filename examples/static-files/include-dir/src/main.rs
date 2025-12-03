#![allow(clippy::unused_async)]

use http_body_util::Full;
use include_dir::{Dir, include_dir};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use vidi::{
    IntoResponse, Request, RequestExt, Response, ResponseExt, Result, Router, StatusCode, serve,
};

const ASSETS: Dir = include_dir!("examples/static-files/include-dir/html"); // frontend dir

pub async fn index(_: Request) -> Result<Response> {
    let file_content = ASSETS
        .get_file("index.html")
        .unwrap()
        .contents_utf8()
        .unwrap();
    Ok(Response::html(file_content))
}

pub async fn assets(req: Request) -> Result<Response> {
    let path = req.path().trim_start_matches('/');
    if path.contains("..") {
        return Ok(StatusCode::FORBIDDEN.into_response());
    }
    Ok(match ASSETS.get_file(path) {
        Some(v) => {
            let data = v.contents();
            Response::builder().body(Full::from(data).into())?
        }
        None => StatusCode::NOT_FOUND.into_response(),
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let app = Router::new()
        .any("/*", |_| async { Ok(StatusCode::NOT_FOUND) })
        .get("/", index)
        .get("/*", assets);

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
