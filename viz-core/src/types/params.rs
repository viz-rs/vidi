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
#[derive(Debug, Clone)]
pub struct Params<T = Vec<(String, String)>>(pub T);

impl<T> AsRef<T> for Params<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Params<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for Params<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

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
    /// Gets single parameter by name.
    ///
    /// # Errors
    ///
    /// Throws a `ParamsError`
    pub fn find<T>(&self, name: &str) -> Result<T, ParamsError>
    where
        T: FromStr,
        T::Err: Display,
    {
        self.iter()
            .find(|p| p.0 == name)
            .ok_or_else(|| {
                let mut s = String::new();
                s.push_str("missing ");
                s.push_str(name);
                s.push_str(" param");
                ParamsError::SingleParse(s)
            })?
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

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.params().map(Params)
    }
}

/// Rejects a parsing error.
#[derive(ThisError, Debug)]
pub enum ParamsError {
    /// Represents pasing single param was failed.
    #[error("{}", .0)]
    SingleParse(String),
    /// Represents pasing multi param was failed.
    #[error(transparent)]
    Parse(#[from] serde::de::value::Error),
    /// Represents params is empty.
    #[error("params is empty")]
    Empty,
}

impl IntoResponse for ParamsError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

impl From<ParamsError> for Error {
    fn from(e: ParamsError) -> Self {
        e.into_error()
    }
}
