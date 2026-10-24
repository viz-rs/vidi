//! Represents a Params extractor for the path of a URL.

mod de;

use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use serde::de::DeserializeOwned;

use crate::{
    Error, FromRequest, IntoResponse, Request, RequestExt, Response, StatusCode, ThisError,
};

#[allow(clippy::redundant_pub_crate)]
pub(crate) use de::PathDeserializer;

/// Extracts params from the path of a URL.
#[derive(Clone, Debug)]
pub struct Params<T = Vec<(String, String)>>(pub T);

impl_deref!(Params);

impl From<Vec<(&str, &str)>> for Params {
    fn from(v: Vec<(&str, &str)>) -> Self {
        Self(
            v.into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        )
    }
}

impl Params {
    /// Returns a single parameter by name.
    ///
    /// # Errors
    ///
    /// Returns a [`ParamsError`] if the parameter is missing or cannot be parsed.
    #[inline]
    pub fn find<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: FromStr,
        T::Err: Display,
    {
        self.iter()
            .find(|p| p.0 == name)
            .ok_or_else(|| ParamsError::SingleParse(format!("missing {name} param")))?
            .1
            .parse()
            .map_err(|e: T::Err| ParamsError::SingleParse(e.to_string()))
    }
}

impl<T> FromRequest for Params<T>
where
    T: DeserializeOwned,
{
    type Error = ParamsError;

    #[allow(clippy::unused_async_trait_impl)]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.params().map(Params)
    }
}

/// Rejects a parsing error.
#[derive(Debug, ThisError)]
pub enum ParamsError {
    /// Failed to parse a single param.
    #[error("{}", .0)]
    SingleParse(String),
    /// Failed to parse multiple params.
    #[error(transparent)]
    Parse(#[from] serde::de::value::Error),
    /// The params are empty.
    #[error("params is empty")]
    Empty,
}

impl_error!(ParamsError);
