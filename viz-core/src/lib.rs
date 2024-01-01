//! The core traits and types in for the [`Viz`].
//!
//! [`Viz`]: https://docs.rs/viz/latest/viz

#![doc(html_logo_url = "https://viz.rs/logo.svg")]
#![doc(html_favicon_url = "https://viz.rs/logo.svg")]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[macro_use]
pub(crate) mod macros;

pub mod handler;
#[doc(inline)]
pub use crate::handler::{BoxHandler, FnExt, Handler, HandlerExt, IntoHandler, Next, Transform};

pub mod middleware;
pub mod types;

mod body;
pub use body::{Body, BodyState};

mod error;
pub use error::{BoxError, Error};

mod from_request;
pub use from_request::FromRequest;

mod into_response;
pub use into_response::IntoResponse;

mod request;
pub use request::RequestExt;

mod response;
pub use response::ResponseExt;

/// Represents an HTTP Request.
pub type Request<T = Body> = http::Request<T>;
/// Represents an HTTP Response.
pub type Response<T = Body> = http::Response<T>;
/// Represents either success (Ok) or failure (Err).
pub type Result<T, E = Error> = ::core::result::Result<T, E>;

pub use async_trait::async_trait;
pub use bytes::{Bytes, BytesMut};
pub use core::future::Future;
pub use futures_util::future;
#[doc(inline)]
pub use headers;
pub use http::{header, Method, StatusCode};
pub use hyper::body::{Body as HttpBody, Incoming};
pub use hyper_util::rt::TokioIo as Io;
pub use thiserror::Error as ThisError;

#[doc(hidden)]
mod tuples {
    use super::{Error, FnExt, FromRequest, Future, IntoResponse, Request, Result};

    tuple_impls!(A B C D E F G H I J K L);
}
