#![allow(clippy::must_use_candidate)]
#![allow(clippy::needless_lifetimes)]
#![allow(clippy::inherent_to_string_shadow_display)]

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
    let body = TodosTemplate { items }.to_string();

    Ok(Response::html(body))
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

markup::define! {
    TodosTemplate<'a>(items: Vec<Todo<'a>>) {
        @markup::doctype()
        html {
            head {
                title { "Todos" }
            }
            body {
                table {
                    tr { th { "ID" } th { "Content" } }
                    @for item in items {
                        tr {
                            td { @item.id }
                            td { @item.content }
                        }
                    }
                }
            }
        }
    }
}
