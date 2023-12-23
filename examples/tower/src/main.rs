//! Viz + Tower services

use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use tower::{
    service_fn,
    util::{MapErrLayer, MapRequestLayer, MapResponseLayer},
    ServiceBuilder,
};
use tower_http::{
    limit::RequestBodyLimitLayer,
    request_id::{MakeRequestUuid, SetRequestIdLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use viz::{serve, Body, Error, IntoResponse, Request, Response, Result, Router, Tree};
use viz_tower::{Layered, ServiceHandler};

async fn index(_: Request) -> Result<Response> {
    Ok("Hello, World!".into_response())
}

async fn about(_: Request) -> Result<&'static str> {
    Ok("About me!")
}

async fn any(_: Request) -> Result<String> {
    Ok("std::any::Any".to_string())
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tower-example=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let index_svc = ServiceBuilder::new()
        .layer(MapRequestLayer::new(|req: Request<_>| req.map(Body::wrap)))
        .service(service_fn(index));
    let index_handler = ServiceHandler::new(index_svc);

    let any_svc = ServiceBuilder::new()
        .layer(MapResponseLayer::new(IntoResponse::into_response))
        .service_fn(any);
    let any_handler = ServiceHandler::new(any_svc);

    let layer = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(1024))
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
        .layer(MapErrLayer::new(Error::from))
        .layer(MapResponseLayer::new(IntoResponse::into_response))
        .layer(MapRequestLayer::new(|req: Request<_>| req.map(Body::wrap)));

    let app = Router::new()
        .get("/", index_handler)
        .get("/about", about)
        .any("/*", any_handler)
        .with(Layered::new(layer));
    let tree = Arc::new(Tree::from(app));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(serve(stream, tree, Some(addr)));
    }
}
