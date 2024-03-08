#![deny(warnings)]

use std::net::SocketAddr;
use tokio::net::TcpListener;

use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    metrics::{self, Aggregation, Instrument, MeterProviderBuilder, Stream},
    Resource,
};

use viz::{
    handlers::prometheus::{ExporterBuilder, Prometheus, Registry},
    middleware::otel,
    serve, Error, Request, Result, Router,
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
    let (exporter, controller) = {
        (
            ExporterBuilder::default()
                .with_registry(registry.clone())
                .build()
                .map_err(Error::boxed)?,
            metrics::new_view(
                Instrument::new().name("http.server.duration"),
                Stream::new().aggregation(Aggregation::ExplicitBucketHistogram {
                    boundaries: vec![
                        0.0, 0.005, 0.01, 0.025, 0.05, 0.075, 0.1, 0.25, 0.5, 0.75, 1.0, 2.5, 5.0,
                        7.5, 10.0,
                    ],
                    record_min_max: true,
                }),
            )
            .unwrap(),
        )
    };
    let provider = MeterProviderBuilder::default()
        .with_reader(exporter)
        .with_resource(Resource::new([KeyValue::new("service.name", "viz")]))
        .with_view(controller)
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

    Ok(())

    // Ensure all spans have been reported
    // global::shutdown_tracer_provider();
    // provider.shutdown();
}
