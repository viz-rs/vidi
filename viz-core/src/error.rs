use std::error::Error as StdError;

use crate::{IntoResponse, Response, StatusCode, ThisError};

/// Represents errors that can occur handling application.
#[derive(ThisError, Debug)]
pub enum Error {
    /// Receives a [`Response`] as error.
    #[error("response")]
    Responder(Response),
    /// Receives a boxed [`std::error::Error`][StdError] as error.
    #[error(transparent)]
    Normal(Box<dyn StdError + Send + Sync>),
    /// Receives a boxed [`std::error::Error`][StdError] and [`Response`] pair as error.
    #[error("report")]
    Report(Box<dyn StdError + Send + Sync>, Response),
}

impl Error {
    /// Create a new error object from any error type.
    pub fn normal<T>(t: T) -> Self
    where
        T: StdError + Send + Sync + 'static,
    {
        Self::Normal(Box::new(t))
    }

    /// Forwards to the method defined on the type `dyn Error`.
    #[inline]
    pub fn is<T>(&self) -> bool
    where
        T: StdError + 'static,
    {
        match self {
            Self::Normal(e) | Self::Report(e, _) => e.is::<T>(),
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
        if let Self::Normal(e) = self {
            return match e.downcast::<T>() {
                Ok(e) => Ok(*e),
                Err(e) => Err(Self::Normal(e)),
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
    #[inline]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: StdError + 'static,
    {
        if let Self::Normal(e) = self {
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
        if let Self::Normal(e) = self {
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
    E: StdError + Send + Sync + 'static,
    T: IntoResponse,
{
    fn from((e, t): (E, T)) -> Self {
        Self::Report(Box::new(e), t.into_response())
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
        Self::normal(e)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Self::normal(e)
    }
}

impl From<Box<dyn StdError + Send + Sync>> for Error {
    fn from(value: Box<dyn StdError + Send + Sync>) -> Self {
        Self::Normal(value)
    }
}
