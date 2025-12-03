<p align="center">
  <img src="https://raw.githubusercontent.com/viz-rs/viz-rs.github.io/gh-pages/logo.svg" height="200" />
</p>

<h1 align="center">
  <a href="https://docs.rs/vidi">Vidi</a>
</h1>

<div align="center">
  <p><strong>Macros for Vidi</strong></p>
</div>

<div align="center">
  <!-- Safety -->
  <a href="/">
    <img src="https://img.shields.io/badge/-safety!-success?style=flat-square"
      alt="Safety!" /></a>
  <!-- Docs.rs docs -->
  <a href="https://docs.rs/vidi-macros">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="Docs.rs docs" /></a>
  <!-- Crates version -->
  <a href="https://crates.io/crates/vidi-macros">
    <img src="https://img.shields.io/crates/v/vidi-macros.svg?style=flat-square"
    alt="Crates.io version" /></a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/vidi-macros">
    <img src="https://img.shields.io/crates/d/vidi-macros.svg?style=flat-square"
      alt="Download" /></a>
</div>

## Macros

| Macro       | Description                      |
| ----------- | -------------------------------- |
| **handler** | Extended Handler with Extractors |

## Example

```rust
use vidi::{IntoResponse, Result, types::{Params}};
use vidi_macros::handler;

#[handler]
fn about() -> impl IntoResponse {
}

#[handler]
async fn index() -> impl IntoResponse {
    ()
}

#[handler]
async fn get_user(Params(name): Params<String>) -> Result<impl IntoResponse> {
    Ok(name)
}
```

## License

This project is licensed under the [MIT license](LICENSE).

## Author

- [@fundon@fosstodon.org](https://fosstodon.org/@fundon)

- [@\_fundon](https://twitter.com/_fundon)
