use std::io;
use std::sync::Arc;

use async_net::TcpListener;
use macro_rules_attribute::apply;
use smol_macros::{Executor, main};
use viz_smol::{IntoResponse, Request, Response, Result, Router};

#[apply(main!)]
async fn main(ex: &Arc<Executor<'_>>) -> io::Result<()> {
    // Build our application with a route.
    let app = Router::new().get("/", handler);

    // Create a `smol`-based TCP listener.
    let listener = TcpListener::bind(("127.0.0.1", 3000)).await.unwrap();
    println!("listening on {}", listener.local_addr().unwrap());

    // Run it
    viz_smol::serve(ex, listener, app).await
}

async fn handler(_: Request) -> Result<Response> {
    Ok("<h1>Hello, World!</h1>".into_response())
}
