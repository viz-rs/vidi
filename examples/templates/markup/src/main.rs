#![deny(warnings)]
#![allow(clippy::unused_async)]

use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use viz::{
    server::conn::http1, BytesMut, Request, Responder, Response, ResponseExt, Result, Router, Tree,
};

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
            id: 1,
            content: "Learn English",
        },
    ];
    let mut buf = BytesMut::with_capacity(512);
    buf.extend(TodosTemplate { items }.to_string().as_bytes());

    Ok(Response::html(buf.freeze()))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {addr}");

    let app = Router::new().get("/", index);
    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, Responder::new(tree, Some(addr)))
                .await
            {
                eprintln!("Error while serving HTTP connection: {err}");
            }
        });
    }
}

markup::define! {
    TodosTemplate<'a>(items: Vec<Todo<'a>>) {
        {markup::doctype()}
        html {
            head {
                title { "Todos" }
            }
            body {
                table {
                    tr { th { "ID" } th { "Content" } }
                    @for item in items {
                        tr {
                            td { {item.id} }
                            td { {markup::raw(v_htmlescape::escape(item.content).to_string())} }
                        }
                    }
                }
            }
        }
    }
}
