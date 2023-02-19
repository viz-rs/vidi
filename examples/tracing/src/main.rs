#![deny(warnings)]

use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use viz::{server::conn::http1, Request, RequestExt, Responder, Result, Router, Tree};
use tracing::{debug, info, instrument, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[instrument]
async fn index(req: Request) -> Result<&'static str> {
    debug!("{} - {}", req.method(), req.path());
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tracing=debug,hyper=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    info!("listening on {addr}");

    let app = Router::new().get("/", index);
    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, Responder::new(tree, Some(addr)))
                .await
            {
                error!("Error while serving HTTP connection: {err}");
            }
        });
    }
}
