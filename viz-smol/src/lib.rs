//! Viz
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
//! # Hello Viz
//!
//! ```no_run
//! use std::io;
//! use std::sync::Arc;
//!
//! use async_net::TcpListener;
//! use macro_rules_attribute::apply;
//! use viz_smol::{Request, Result, Router};
//!
//! async fn index(_: Request) -> Result<&'static str> {
//!     Ok("Hello, Viz!")
//! }
//!
//! #[apply(smol_macros::main!)]
//! async fn main(ex: &Arc<smol_macros::Executor<'_>>) -> io::Result<()> {
//!     // Build our application with a route.
//!     let app = Router::new().get("/", index);
//!
//!     // Create a `smol`-based TCP listener.
//!     let listener = TcpListener::bind(("127.0.0.1", 3000)).await.unwrap();
//!     println!("listening on {}", listener.local_addr().unwrap());
//!
//!     // Run it
//!     viz_smol::serve(ex.clone(), listener, app).await
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
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

mod responder;
pub use responder::Responder;

mod listener;
pub use listener::Listener;

mod server;
pub use server::serve;

// #[cfg(any(feature = "native-tls", feature = "rustls"))]
// pub use server::tls;

pub use viz_core::*;
pub use viz_router::*;

#[cfg(feature = "handlers")]
#[cfg_attr(docsrs, doc(cfg(feature = "handlers")))]
#[doc(inline)]
pub use viz_handlers as handlers;

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[doc(inline)]
pub use viz_macros::handler;
