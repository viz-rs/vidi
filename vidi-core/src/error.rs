use std::error::Error as StdError;

use crate::{IntoResponse, Response, StatusCode, ThisError};

/// A boxed [`StdError`].
pub type BoxError = Box<dyn StdError + Send + Sync>;

/// Errors that can occur while handling a request.
#[derive(Debug, ThisError)]
pub enum Error {
    /// A boxed [`std::error::Error`][StdError].
    #[error(transparent)]
    Boxed(BoxError),
    /// A [`Response`] used as an error.
    #[error("response")]
    Response(Box<Response>),
    /// A boxed [`std::error::Error`][StdError] paired with a [`Response`].
    #[error("{0}")]
    WithResponse(BoxError, Box<Response>),
}

impl Error {
    /// Creates an [`Error`] from any error type.
    pub fn boxed<T>(t: T) -> Self
    where
        T: Into<BoxError>,
    {
        Self::Boxed(t.into())
    }

    /// Returns whether the inner error is of type `T`.
    #[must_use]
    #[inline]
    pub fn is<T>(&self) -> bool
    where
        T: StdError + 'static,
    {
        match self {
            Self::Boxed(e) | Self::WithResponse(e, _) => e.is::<T>(),
            Self::Response(_) => false,
        }
    }

    /// Downcasts the error to a concrete type.
    ///
    /// # Errors
    ///
    /// Returns an `Error` if the downcast fails.
    #[inline]
    pub fn downcast<T>(self) -> Result<T, Self>
    where
        T: StdError + 'static,
    {
        if let Self::Boxed(e) = self {
            return match e.downcast::<T>() {
                Ok(e) => Ok(*e),
                Err(e) => Err(Self::Boxed(e)),
            };
        }
        if let Self::WithResponse(e, r) = self {
            return match e.downcast::<T>() {
                Ok(e) => Ok(*e),
                Err(e) => Err(Self::WithResponse(e, r)),
            };
        }
        Err(self)
    }

    /// Downcasts the error by reference.
    #[must_use]
    #[inline]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: StdError + 'static,
    {
        if let Self::Boxed(e) = self {
            return e.downcast_ref::<T>();
        }
        if let Self::WithResponse(e, _) = self {
            return e.downcast_ref::<T>();
        }
        None
    }

    /// Downcasts the error by mutable reference.
    #[inline]
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: StdError + 'static,
    {
        if let Self::Boxed(e) = self {
            return e.downcast_mut::<T>();
        }
        if let Self::WithResponse(e, _) = self {
            return e.downcast_mut::<T>();
        }
        None
    }
}

impl<E, T> From<(E, T)> for Error
where
    E: Into<BoxError>,
    T: IntoResponse,
{
    fn from((e, t): (E, T)) -> Self {
        Self::WithResponse(e.into(), Box::new(t.into_response()))
    }
}

impl From<http::Error> for Error {
    fn from(e: http::Error) -> Self {
        (e, StatusCode::BAD_REQUEST).into()
    }
}

impl From<hyper::Error> for Error {
    fn from(e: hyper::Error) -> Self {
        (e, StatusCode::BAD_REQUEST).into()
    }
}

impl From<std::convert::Infallible> for Error {
    fn from(e: std::convert::Infallible) -> Self {
        match e {}
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::boxed(e)
    }
}

impl From<BoxError> for Error {
    fn from(value: BoxError) -> Self {
        Self::Boxed(value)
    }
}
