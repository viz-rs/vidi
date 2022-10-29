#![deny(warnings)]

use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;

use sessions::MemoryStorage;

use viz::{
    get,
    middleware::{
        cookie,
        helper::CookieOptions,
        session::{self, Store},
    },
    server::conn::http1,
    Request, RequestExt, Responder, Result, Router, Tree,
};

async fn index(req: Request) -> Result<&'static str> {
    req.session().set(
        "counter",
        req.session().get::<u64>("counter")?.unwrap_or_default() + 1,
    )?;
    Ok("Hello, World!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {addr}");

    let app = Router::new()
        .route("/", get(index))
        .with(session::Config::new(
            Store::new(MemoryStorage::new(), nano_id::base64::<32>, |sid: &str| {
                sid.len() == 32
            }),
            CookieOptions::default(),
        ))
        .with(cookie::Config::new());
    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, Responder::new(tree, Some(addr)))
                .await
            {
                eprintln!("Error while serving HTTP connection: {}", err);
            }
        });
    }
}
