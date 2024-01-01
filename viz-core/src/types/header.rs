//! Represents a header extractor.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use crate::{
    header,
    headers::{self, HeaderMapExt},
    Error, FromRequest, IntoResponse, Request, Response, Result, StatusCode, ThisError,
};

/// Extracts a header from the headers of a request.
pub struct Header<T: ?Sized>(pub T);

impl<T> Header<T> {
    /// Create new `Header` instance.
    #[inline]
    pub fn new(t: T) -> Self {
        Self(t)
    }

    /// Consumes the Header, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Clone for Header<T>
where
    T: ?Sized + Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> AsRef<T> for Header<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Header<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Header<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Debug for Header<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

#[crate::async_trait]
impl<T> FromRequest for Header<T>
where
    T: headers::Header,
{
    type Error = HeaderError;

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.headers()
            .typed_try_get::<T>()
            .map_err(|_| HeaderError::InvalidName(T::name()))
            .and_then(|v| v.ok_or_else(|| HeaderError::MissingName(T::name())))
            .map(Self)
    }
}

/// Rejects with an error when header extraction fails.
#[derive(Debug, ThisError)]
pub enum HeaderError {
    /// Invalid header name.
    #[error("Invalid header name {0}")]
    InvalidName(&'static header::HeaderName),
    /// Missing header name.
    #[error("Missing header name {0}")]
    MissingName(&'static header::HeaderName),
    /// Missing header value.
    #[error("Invalid header value {0}")]
    InvalidValue(header::InvalidHeaderValue),
}

impl IntoResponse for HeaderError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

impl From<HeaderError> for Error {
    fn from(e: HeaderError) -> Self {
        e.into_error()
    }
}
