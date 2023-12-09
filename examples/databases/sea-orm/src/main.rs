#![deny(warnings)]
#![allow(clippy::unused_async)]

//! `SeaOrm` example for Viz framework.
use sea_orm_example::{api, db::init_db};
use std::{env, net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use viz::{handlers::serve, middleware, serve, types::State, Result, Router, Tree};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;

    let db = init_db().await?;

    println!("listening on http://{addr}");
    let dir = env::current_dir().unwrap();
    let app = Router::new()
        .get("/", serve::File::new(dir.join("public/index.html")))
        .get("/todos", api::list)
        .post("/todos", api::create)
        .put("/todos/:id", api::update)
        .delete("/todos/:id", api::delete)
        .with(State::new(db.clone()))
        .with(middleware::limits::Config::new());
    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(serve(stream, tree, Some(addr)));
    }
}
