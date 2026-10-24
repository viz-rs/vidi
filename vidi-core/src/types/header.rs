//! Typed header extractor.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use crate::{
    Error, FromRequest, IntoResponse, Request, Response, Result, StatusCode, ThisError, header,
    headers::HeaderMapExt,
};

/// Extracts a typed header from the request.
pub struct Header<T>(pub T);

impl_newtype!(Header, "header");

impl<T> FromRequest for Header<T>
where
    T: headers::Header,
{
    type Error = HeaderError;

    #[allow(clippy::unused_async_trait_impl)]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.headers()
            .typed_try_get::<T>()
            .map_err(|_| HeaderError::InvalidName(T::name()))
            .and_then(|v| v.ok_or_else(|| HeaderError::MissingName(T::name())))
            .map(Self)
    }
}

/// Error returned when header extraction fails.
#[derive(Debug, ThisError)]
pub enum HeaderError {
    /// Invalid header name.
    #[error("Invalid header name {0}")]
    InvalidName(&'static header::HeaderName),
    /// Missing header name.
    #[error("Missing header name {0}")]
    MissingName(&'static header::HeaderName),
    /// Invalid header value.
    #[error("Invalid header value {0}")]
    InvalidValue(header::InvalidHeaderValue),
}

impl_error!(HeaderError);
