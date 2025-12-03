//! Request Form Body Extractor

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use serde::de::DeserializeOwned;

use crate::{FromRequest, Request, RequestExt, Result};

use super::{Payload, PayloadError};

/// Extracts from-data from the body of a request.
pub struct Form<T = ()>(pub T);

impl<T> Form<T> {
    /// Create new `Form` instance.
    #[inline]
    pub const fn new(data: T) -> Self {
        Self(data)
    }

    /// Consumes the Form, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Clone for Form<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> AsRef<T> for Form<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Form<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Form<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Debug for Form<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

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
