#![deny(warnings)]

use opentelemetry::{
    global,
    runtime::TokioCurrentThread,
    sdk::{propagation::TraceContextPropagator, trace::Tracer},
};
use std::net::SocketAddr;
use viz::{middleware::otel, Request, Result, Router, Server, ServiceMaker};

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
    println!("listening on {}", addr);

    let tracer = init_tracer();

    let app = Router::new()
        .get("/", index)
        .get("/:username", index)
        .with(otel::tracing::Config::new(tracer));

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
