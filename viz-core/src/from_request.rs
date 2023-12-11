//! Extracts data from the [`Request`] by types.

use crate::{Future, IntoResponse, Request};

/// An interface for extracting data from the HTTP [`Request`].
pub trait FromRequest: Sized {
    /// The type returned in the event of a conversion error.
    type Error: IntoResponse;

    /// Extracts this type from the HTTP [`Request`].
    #[must_use]
    fn extract(req: &mut Request) -> impl Future<Output = Result<Self, Self::Error>> + Send;
}

impl<T> FromRequest for Option<T>
where
    T: FromRequest,
{
    type Error = std::convert::Infallible;

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(T::extract(req).await.ok())
    }
}

impl<T> FromRequest for Result<T, T::Error>
where
    T: FromRequest,
{
    type Error = std::convert::Infallible;

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        Ok(T::extract(req).await)
    }
}
