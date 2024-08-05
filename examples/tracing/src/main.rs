use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{debug, error, info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use viz::{serve, Request, RequestExt, Result, Router};

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
                .unwrap_or_else(|_| "trace,tracing=debug,hyper=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    info!("listening on http://{addr}");

    let app = Router::new().get("/", index);

    if let Err(e) = serve(listener, app).await {
        error!("{e}");
    }

    Ok(())
}
