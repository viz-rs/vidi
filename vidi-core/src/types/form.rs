//! Form body extractor.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use serde::de::DeserializeOwned;

use crate::{FromRequest, Request, RequestExt, Result};

use super::{Payload, PayloadError};

/// Extracts form data from the request body.
pub struct Form<T = ()>(pub T);

impl_newtype!(Form, "form");

impl<T> Payload for Form<T> {
    const NAME: &'static str = "form";

    // 32KB
    const LIMIT: u64 = 1024 * 32;

    fn detect(m: &mime::Mime) -> bool {
        m.type_() == mime::APPLICATION && m.subtype() == mime::WWW_FORM_URLENCODED
    }

    fn mime() -> mime::Mime {
        mime::APPLICATION_WWW_FORM_URLENCODED
    }
}

impl<T> FromRequest for Form<T>
where
    T: DeserializeOwned,
{
    type Error = PayloadError;

    #[inline]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.form().await.map(Self)
    }
}
