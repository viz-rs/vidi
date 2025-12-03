use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use vidi::{Request, Result, Router, get, serve, tls};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let app = Router::new().route("/", get(index));

    let listener = tls::TlsListener::<_, tls::rustls::TlsAcceptor>::new(
        listener,
        tls::rustls::Config::new()
            .cert(include_bytes!("../../tls/cert.pem").to_vec())
            .key(include_bytes!("../../tls/key.pem").to_vec())
            .build()
            .map(Arc::new)?
            .into(),
    );

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
