//! Unix Domain Socket
//!
//! ```sh
//! curl --unix-socket /tmp/viz.sock http://localhost/
//! ```
#![deny(warnings)]

#[cfg(unix)]
#[tokio::main]
async fn main() -> viz::Result<()> {
    use tokio::net::UnixListener;
    use viz::{get, serve, IntoHandler, Result, Router};

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
