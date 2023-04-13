<p align="center">
  <img src="https://raw.githubusercontent.com/viz-rs/viz-rs.github.io/gh-pages/logo.svg" height="200" />
</p>

<h1 align="center">
  <a href="https://viz.rs">Viz</a>
</h1>

<div align="center">
  <p><strong>Fast, robust, flexible, lightweight web framework for Rust</strong></p>
</div>

<div align="center">
  <!-- Safety -->
  <a href="/">
    <img src="https://img.shields.io/badge/-safety!-success?style=flat-square"
      alt="Safety!" /></a>
  <!-- Docs.rs docs -->
  <a href="https://docs.rs/viz">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="Docs.rs docs" /></a>
  <!-- Crates version -->
  <a href="https://crates.io/crates/viz">
    <img src="https://img.shields.io/crates/v/viz.svg?style=flat-square"
    alt="Crates.io version" /></a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/viz">
    <img src="https://img.shields.io/crates/d/viz.svg?style=flat-square"
      alt="Download" /></a>
  <!-- Codecov -->
  <a href="/">
    <img src="https://img.shields.io/codecov/c/github/viz-rs/viz?style=flat-square"
      alt="Codecov" /></a>
  <!-- Discord -->
  <a href="https://discord.gg/m9yAsf6jg6">
     <img src="https://img.shields.io/discord/699908392105541722?logo=discord&style=flat-square"
     alt="Discord"></a>
</div>

> **Note**: viz's [main](https://github.com/viz-rs/viz) branch is
> currently preparing breaking changes. For the most recently *released* code,
> look to the [0.4.x branch](https://github.com/viz-rs/viz/tree/0.4.x).

## Features

- **Safety** `#![forbid(unsafe_code)]`

- Lightweight

- Robust `Routing`

- Handy `Extractors`

- Simple + Flexible `Handler` & `Middleware`

## Hello Viz

```rust
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use viz::{server::conn::http1, Request, Responder, Result, Router, Tree};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, Viz!")
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
```

More examples can be found
[here](https://github.com/viz-rs/viz/tree/main/examples).

## Get started

* [English](https://viz.rs)

* [简体中文](https://zh-cn.viz.rs)

## License

This project is licensed under the [MIT license](LICENSE).

## Author

- [@fundon@fosstodon.org](https://fosstodon.org/@fundon)

- [@\_fundon](https://twitter.com/_fundon)
