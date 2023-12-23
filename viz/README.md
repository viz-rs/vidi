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
  <a href="https://app.codecov.io/gh/viz-rs/viz">
    <img src="https://img.shields.io/codecov/c/github/viz-rs/viz?style=flat-square"
      alt="Codecov" /></a>
  <!-- Discord -->
  <a href="https://discord.gg/m9yAsf6jg6">
     <img src="https://img.shields.io/discord/699908392105541722?logo=discord&style=flat-square"
     alt="Discord"></a>
</div>

## Features

- **Safety** `#![forbid(unsafe_code)]`

- Lightweight

- Robust `Routing`

- Handy `Extractors`

- Simple + Flexible `Handler` & `Middleware`

- Supports: Tower `Service`

## Hello Viz

```rust
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use viz::{serve, Request, Result, Router, Tree};

async fn index(_: Request) -> Result<&'static str> {
    Ok("Hello, Viz!")
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let app = Router::new().get("/", index);
    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(serve(stream, tree, Some(addr)));
    }
}
```

More examples can be found
[here](https://github.com/viz-rs/viz/tree/main/examples).

## Get started

Open [Viz.rs](https://viz.rs), select language or version.

## License

This project is licensed under the [MIT license](LICENSE).

## Author

- [@fundon@fosstodon.org](https://fosstodon.org/@fundon)

- [@\_fundon](https://twitter.com/_fundon)
