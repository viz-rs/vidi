use std::error::Error as StdError;

use crate::{IntoResponse, Response, StatusCode, ThisError};

/// An owned dynamically typed [`StdError`].
pub type BoxError = Box<dyn StdError + Send + Sync>;

/// Represents errors that can occur handling application.
#[derive(Debug, ThisError)]
pub enum Error {
    /// Receives a boxed [`std::error::Error`][StdError] as an error.
    #[error(transparent)]
    Boxed(BoxError),
    /// Receives a [`Response`] as an error.
    #[error("response")]
    Responder(Box<Response>),
    /// Receives a boxed [`std::error::Error`][StdError] and [`Response`] pair as an error.
    #[error("report")]
    Report(BoxError, Box<Response>),
}

impl Error {
    /// Create a new error object from any error type.
    pub fn boxed<T>(t: T) -> Self
    where
        T: Into<BoxError>,
    {
        Self::Boxed(t.into())
    }

    /// Forwards to the method defined on the type `dyn Error`.
    #[must_use]
    #[inline]
    pub fn is<T>(&self) -> bool
    where
        T: StdError + 'static,
    {
        match self {
            Self::Boxed(e) | Self::Report(e, _) => e.is::<T>(),
            Self::Responder(_) => false,
        }
    }

    /// Attempt to downcast the error object to a concrete type.
    ///
    /// # Errors
    ///
    /// Throws an `Error` if downcast fails.
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
        if let Self::Report(e, r) = self {
            return match e.downcast::<T>() {
                Ok(e) => Ok(*e),
                Err(e) => Err(Self::Report(e, r)),
            };
        }
        Err(self)
    }

    /// Downcast this error object by reference.
    #[must_use]
    #[inline]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: StdError + 'static,
    {
        if let Self::Boxed(e) = self {
            return e.downcast_ref::<T>();
        }
        if let Self::Report(e, _) = self {
            return e.downcast_ref::<T>();
        }
        None
    }

    /// Downcast this error object by mutable reference.
    #[inline]
    pub fn downcast_mut<T>(&mut self) -> Option<&mut T>
    where
        T: StdError + 'static,
    {
        if let Self::Boxed(e) = self {
            return e.downcast_mut::<T>();
        }
        if let Self::Report(e, _) = self {
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
        Self::Report(e.into(), Box::new(t.into_response()))
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
        Self::boxed(e)
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
