use crate::{FromRequest, IntoResponse, Request, Result};

use super::{FnExt, FnExtHandler, Handler};

/// Trait implemented by types that can be converted to a [`Handler`].
pub trait IntoHandler<E, I> {
    /// The target handler.
    type Handler: Handler<I>;

    /// Convert self to a [Handler].
    #[must_use]
    fn into_handler(self) -> Self::Handler;
}

impl<H, E, O> IntoHandler<E, Request> for H
where
    E: FromRequest + Send + Sync + 'static,
    E::Error: IntoResponse + Send + Sync,
    H: FnExt<E, Output = Result<O>>,
    O: Send + Sync + 'static,
{
    type Handler = FnExtHandler<H, E, O>;

    fn into_handler(self) -> Self::Handler {
        FnExtHandler::new(self)
    }
}
