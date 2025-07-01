//! Unix Domain Socket
//!
//! ```sh
//! curl --unix-socket /tmp/viz.sock http://localhost/
//! ```

#[cfg(unix)]
#[tokio::main]
async fn main() -> viz::Result<()> {
    use tokio::net::UnixListener;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    use viz::{IntoHandler, Result, Router, get, serve};

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug,tracing=debug,hyper=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    async fn index() -> Result<&'static str> {
        Ok("Hello world!")
    }

    let path = "/tmp/viz.sock";
    println!("listening on http://{path}");

    let listener = UnixListener::bind(path)?;

    let app = Router::new().route("/", get(index.into_handler()));

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}

#[cfg(not(unix))]
#[tokio::main]
async fn main() {
    panic!("Must run under Unix-like platform!");
}
