use std::{env, net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use viz::{handlers::serve, serve, Request, Response, ResponseExt, Result, Router};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();

    let app = Router::new()
        .get("/", index)
        .get("/cargo.toml", serve::File::new(dir.join("Cargo.toml")))
        .get(
            "/examples/*",
            serve::Dir::new(dir.join("../../../examples")).listing(),
        )
        .any("/*", |_| async { Ok(Response::text("Welcome!")) });

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
