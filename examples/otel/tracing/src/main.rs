#![deny(warnings)]

use opentelemetry::{
    global,
    runtime::TokioCurrentThread,
    sdk::{propagation::TraceContextPropagator, trace::Tracer},
};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use viz::{middleware::otel, server::conn::http1, Request, Responder, Result, Router, Tree};

fn init_tracer() -> Tracer {
    global::set_text_map_propagator(TraceContextPropagator::new());
    opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("viz")
        .install_batch(TokioCurrentThread)
        .unwrap()
}

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {addr}");

    let tracer = init_tracer();

    let app = Router::new()
        .get("/", index)
        .get("/:username", index)
        .with(otel::tracing::Config::new(tracer));
    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, Responder::new(tree, Some(addr)))
                .await
            {
                eprintln!("Error while serving HTTP connection: {}", err);
            }
        });
    }
}
