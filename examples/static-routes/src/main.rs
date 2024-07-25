#![deny(warnings)]

use hyper::server::conn::http1;
use hyper::service::service_fn;
use std::{
    net::SocketAddr,
    sync::{Arc, LazyLock},
};
use tokio::net::TcpListener;

use viz::{
    types::{Params, RouteInfo},
    Body, Handler, IntoResponse, Io, Method, Request, RequestExt, Response, Result, Router,
    StatusCode, Tree,
};

/// Static Lazy Routes
static TREE: LazyLock<Tree> = LazyLock::new(|| Router::new().get("/", index).get("/me", me).into());

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

async fn me(_: Request) -> Result<&'static str> {
    Ok("Hi, It's me!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    loop {
        let (stream, addr) = listener.accept().await?;
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    Io::new(stream),
                    service_fn(|req| serve(req.map(Body::Incoming), Some(addr))),
                )
                .await
            {
                eprintln!("Error while serving HTTP connection: {err}");
            }
        });
    }
}

/// Serves a request and returns a response.
async fn serve(mut req: Request, mut addr: Option<SocketAddr>) -> Result<Response, hyper::Error> {
    let method = req.method().clone();
    let path = req.path().to_owned();
    let responded = Ok(
        match TREE.find(&method, &path).or_else(|| {
            if method == Method::HEAD {
                TREE.find(&Method::GET, &path)
            } else {
                None
            }
        }) {
            Some((handler, route)) => {
                if addr.is_some() {
                    req.extensions_mut().insert(addr.take());
                }
                req.extensions_mut().insert(Arc::from(RouteInfo {
                    id: *route.id,
                    pattern: route.pattern(),
                    params: Into::<Params>::into(route.params()),
                }));
                handler
                    .call(req)
                    .await
                    .unwrap_or_else(IntoResponse::into_response)
            }
            None => StatusCode::NOT_FOUND.into_response(),
        },
    );
    responded
}
