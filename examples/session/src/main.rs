use std::net::SocketAddr;
use tokio::net::TcpListener;

use sessions::MemoryStorage;

use vidi::{
    Request, RequestExt, Result, Router, get,
    middleware::{
        cookie,
        helper::CookieOptions,
        session::{self, Store},
    },
    serve,
    types::CookieKey,
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
    println!("listening on http://{addr}");

    let app = Router::new()
        .route("/", get(index))
        .with(session::Config::new(
            Store::new(MemoryStorage::new(), nano_id::base64::<32>, |sid: &str| {
                sid.len() == 32
            }),
            CookieOptions::default(),
        ))
        .with(cookie::Config::with_key(CookieKey::generate()));

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
