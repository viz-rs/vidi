//! Vidi
//!
//! Fast, robust, flexible, lightweight web framework for Rust.
//!
//! # Features
//!
//! * **Safety** `#![forbid(unsafe_code)]`
//!
//! * Lightweight
//!
//! * Simple + Flexible [`Handler`](#handler) & [`Middleware`](#middleware)
//!
//! * Handy [`Extractors`](#extractors)
//!
//! * Robust [`Routing`](#routing)
//!
//! * Supports Tower [`Service`]
//!
//! # Hello Vidi
//!
//! ```no_run
//! use std::io;
//! use std::sync::Arc;
//!
//! use async_net::TcpListener;
//! use macro_rules_attribute::apply;
//! use smol_macros::{Executor, main};
//! use vidi_smol::{Request, Result, Router};
//!
//! async fn index(_: Request) -> Result<&'static str> {
//!     Ok("Hello, Vidi!")
//! }
//!
//! #[apply(main!)]
//! async fn main(ex: &Arc<Executor<'_>>) -> io::Result<()> {
//!     // Build our application with a route.
//!     let app = Router::new().get("/", index);
//!
//!     // Create a `smol`-based TCP listener.
//!     let listener = TcpListener::bind(("127.0.0.1", 3000)).await.unwrap();
//!     println!("listening on {}", listener.local_addr().unwrap());
//!
//!     // Run it
//!     vidi_smol::serve(ex, listener, app).await
//! }
//! ```
//!
//! [`Service`]: https://docs.rs/tower-service/latest/tower_service/trait.Service.html

#![doc(html_logo_url = "https://viz.rs/logo.svg")]
#![doc(html_favicon_url = "https://viz.rs/logo.svg")]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod responder;
pub use responder::Responder;

mod listener;
pub use listener::Listener;

mod server;
pub use server::serve;

// #[cfg(any(feature = "native-tls", feature = "rustls"))]
// pub use server::tls;

pub use vidi_core::*;
pub use vidi_router::*;

#[cfg(feature = "handlers")]
#[cfg_attr(docsrs, doc(cfg(feature = "handlers")))]
#[doc(inline)]
pub use vidi_handlers as handlers;

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[doc(inline)]
pub use vidi_macros::handler;
