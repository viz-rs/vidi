use std::net::SocketAddr;
use tokio::net::TcpListener;

use opentelemetry::{KeyValue, global};
use opentelemetry_sdk::{Resource, metrics::MeterProviderBuilder};

use viz::{
    Error, Request, Result, Router,
    handlers::prometheus::{ExporterBuilder, Prometheus, Registry},
    middleware::otel,
    serve,
};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let registry = Registry::new();
    let exporter = ExporterBuilder::default()
        .with_registry(registry.clone())
        .build()
        .map_err(Error::boxed)?;
    let provider = MeterProviderBuilder::default()
        .with_reader(exporter)
        .with_resource(
            Resource::builder_empty()
                .with_attributes([KeyValue::new("service.name", "viz")])
                .build(),
        )
        .build();

    global::set_meter_provider(provider.clone());

    let app = Router::new()
        .get("/", index)
        .get("/:username", index)
        .get("/metrics", Prometheus::new(registry))
        .with(otel::metrics::Config::new(&global::meter("otel")));

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    // Ensure all spans have been reported
    provider.shutdown().map_err(Error::boxed)?;

    Ok(())
}
