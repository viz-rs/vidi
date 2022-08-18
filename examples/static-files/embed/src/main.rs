#![deny(warnings)]

use std::net::SocketAddr;
use viz::{handlers::embed, IntoResponse, Result, Router, Server, ServiceMaker, StatusCode};

#[derive(rust_embed::RustEmbed)]
#[folder = "public"]
struct Asset;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let app = Router::new()
        .get("/", embed::File::<Asset>::new("index.html"))
        .get("/static/*", embed::Dir::<Asset>::default())
        .any("/*", |_| async {
            Ok(StatusCode::NOT_FOUND.into_response())
        });

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
