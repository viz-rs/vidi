//! Represents a JSON extractor or responder.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use crate::{FromRequest, IntoResponse, Request, RequestExt, Response, ResponseExt, Result};

use super::{Payload, PayloadError};

/// Extracts JSON data from the body of a request, or responds a JSON data to response.
pub struct Json<T = ()>(pub T);

impl<T> Json<T> {
    /// Create new `Json` instance.
    #[inline]
    pub fn new(data: T) -> Self {
        Json(data)
    }

    /// Consumes the JSON, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Clone for Json<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Json(self.0.clone())
    }
}

impl<T> AsRef<T> for Json<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Debug for Json<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

impl<T> Payload for Json<T> {
    const NAME: &'static str = "json";

    // 1MB
    const LIMIT: u64 = 1024 * 1024;

    fn detect(m: &mime::Mime) -> bool {
        m.type_() == mime::APPLICATION
            && (m.subtype() == mime::JSON || m.suffix() == Some(mime::JSON))
    }

    fn mime() -> mime::Mime {
        mime::APPLICATION_JAVASCRIPT_UTF_8
    }
}

#[crate::async_trait]
impl<T> FromRequest for Json<T>
where
    T: serde::de::DeserializeOwned,
{
    type Error = PayloadError;

    #[inline]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.json().await.map(Self)
    }
}

/// Responds with JSON Data.
impl<T> IntoResponse for Json<T>
where
    T: serde::Serialize,
{
    fn into_response(self) -> Response {
        match Response::json(self.0) {
            Ok(res) => res,
            Err(err) => err.into_response(),
        }
    }
}
