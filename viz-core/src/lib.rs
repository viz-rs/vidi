//! The core traits and types in for the [`Viz`].
//!
//! [`Viz`]: https://docs.rs/viz/latest/viz

#![doc(html_logo_url = "https://viz.rs/logo.svg")]
#![doc(html_favicon_url = "https://viz.rs/logo.svg")]
// #![forbid(unsafe_code)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms),
        allow(dead_code, unused_assignments, unused_variables)
    )
))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

/// Represents an HTTP Request.
pub type Request<T = IncomingBody> = http::Request<T>;
/// Represents an HTTP Response.
pub type Response<T = OutgoingBody> = http::Response<T>;
/// Represents either success (Ok) or failure (Err).
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[macro_use]
pub(crate) mod macros;

pub mod handler;

#[doc(inline)]
pub use crate::handler::{BoxHandler, FnExt, Handler, HandlerExt, IntoHandler, Next, Transform};

pub mod middleware;
pub mod types;

mod body;
mod error;
mod from_request;
mod into_response;
mod request;
mod response;

pub use body::{IncomingBody, OutgoingBody};
pub use error::Error;
pub use from_request::FromRequest;
pub use into_response::IntoResponse;
pub use request::RequestExt;
pub use response::ResponseExt;

pub use async_trait::async_trait;
pub use bytes::{Bytes, BytesMut};
#[doc(inline)]
pub use headers;
pub use http::{header, Method, StatusCode};
pub use hyper::body::{Body, Incoming};
pub use std::future::Future;
pub use thiserror::Error as ThisError;

#[doc(hidden)]
mod tuples {
    use super::{async_trait, Error, FnExt, FromRequest, Future, IntoResponse, Request, Result};

    tuple_impls!(A B C D E F G H I J K L);
}
