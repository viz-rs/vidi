//! `SeaOrm` example for Vidi framework.
use sea_orm_example::{api, db::init_db};
use std::{env, net::SocketAddr, path::PathBuf};
use tokio::net::TcpListener;
use vidi::{Result, Router, handlers::serve, middleware, serve, types::State};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    let db = init_db().await?;

    println!("listening on http://{addr}");

    let dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();

    let app = Router::new()
        .get("/", serve::File::new(dir.join("public/index.html")))
        .get("/todos", api::list)
        .post("/todos", api::create)
        .put("/todos/:id", api::update)
        .delete("/todos/:id", api::delete)
        .with(State::new(db))
        .with(middleware::limits::Config::new());

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
