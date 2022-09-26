#![deny(warnings)]

use std::net::SocketAddr;

use opentelemetry::{
    global,
    sdk::{
        export::metrics::aggregation,
        metrics::{controllers, processors, selectors},
    },
};

use viz::{
    handlers::prometheus::{ExporterBuilder, Prometheus},
    middleware::otel,
    Request, Result, Router, Server, ServiceMaker,
};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let exporter = {
        let controller = controllers::basic(
            processors::factory(
                selectors::simple::histogram([1.0, 2.0, 5.0, 10.0, 20.0, 50.0]),
                aggregation::cumulative_temporality_selector(),
            )
            .with_memory(true),
        )
        .build();
        ExporterBuilder::new(controller).init()
    };

    let meter = global::meter("viz");

    let app = Router::new()
        .get("/", index)
        .get("/:username", index)
        .get("/metrics", Prometheus::new(exporter))
        .with(otel::metrics::Config::new(meter));

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
