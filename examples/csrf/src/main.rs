#![deny(warnings)]

use std::{net::SocketAddr, time::Duration};

use viz::{
    middleware::{
        cookie,
        csrf::{self, CsrfToken},
        helper::CookieOptions,
    },
    Method, Request, RequestExt, Result, Router, Server, ServiceMaker,
};

async fn index(mut req: Request) -> Result<String> {
    Ok(req.extract::<CsrfToken>().await?.0)
}

async fn create(_req: Request) -> Result<&'static str> {
    Ok("CSRF Protection!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("listening on {}", addr);

    let app = Router::new()
        .get("/", index)
        .post("/", create)
        .with(csrf::Config::new(
            csrf::Store::Cookie,
            [Method::GET, Method::HEAD, Method::OPTIONS, Method::TRACE].into(),
            CookieOptions::new("_csrf").max_age(Duration::from_secs(3600 * 24)),
            csrf::secret,
            csrf::generate,
            csrf::verify,
        ))
        .with(cookie::Config::new());

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{}", err);
    }

    Ok(())
}
