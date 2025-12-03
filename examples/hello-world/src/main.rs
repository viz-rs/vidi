use std::{net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;
use vidi::{Request, Result, Router, serve};

async fn index(_: Request) -> Result<String> {
    Ok(String::from("Hello, World!"))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from_str("[::1]:3000").unwrap();
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let mut app = Router::new().get("/", |_| async { Ok("Hello, World!") });

    for n in 0..1000 {
        app = app.get(format!("/{n}"), index);
    }

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
