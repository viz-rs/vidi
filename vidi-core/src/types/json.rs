//! JSON extractor and responder.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use crate::{FromRequest, IntoResponse, Request, RequestExt, Response, ResponseExt, Result};

use super::{Payload, PayloadError};

/// Extracts JSON from the request body, or responds with JSON.
pub struct Json<T = ()>(pub T);

impl_newtype!(Json, "JSON");

impl<T> Payload for Json<T> {
    const NAME: &'static str = "json";

    // 1MB
    const LIMIT: u64 = 1024 * 1024;

    fn detect(m: &mime::Mime) -> bool {
        m.type_() == mime::APPLICATION
            && (m.subtype() == mime::JSON || m.suffix() == Some(mime::JSON))
    }

    fn mime() -> mime::Mime {
        mime::APPLICATION_JSON
    }
}

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

/// Responds with JSON.
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
