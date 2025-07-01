#![allow(clippy::must_use_candidate)]
#![allow(clippy::inherent_to_string_shadow_display)]

use maud::{DOCTYPE, PreEscaped, html};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use viz::{Request, Response, ResponseExt, Result, Router, serve};

pub struct Todo<'a> {
    id: u64,
    content: &'a str,
}

async fn index(_: Request) -> Result<Response> {
    let items = vec![
        Todo {
            id: 1,
            content: "Learn Rust",
        },
        Todo {
            id: 2,
            content: "Learn English",
        },
    ];

    let buf = html! {
        (DOCTYPE)
        head {
            title { "Todos" }
        }
        body {
            table {
                tr { th { "ID" } th { "Content" } }
                @for item in &items {
                    tr {
                        td { (item.id) }
                        td { (PreEscaped(item.content.to_string())) }
                    }
                }
            }
        }
    };

    Ok(Response::html(buf.into_string()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let app = Router::new().get("/", index);

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
