//! Query-string extractor.

use std::{
    fmt,
    ops::{Deref, DerefMut},
};

use serde::de::DeserializeOwned;

use crate::{FromRequest, Request, RequestExt, Result, types::PayloadError};

/// Extracts data from the query string.
pub struct Query<T = ()>(pub T);

impl_newtype!(Query, "query");

impl<T> FromRequest for Query<T>
where
    T: DeserializeOwned,
{
    type Error = PayloadError;

    #[allow(clippy::unused_async_trait_impl)]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.query().map(Self)
    }
}
