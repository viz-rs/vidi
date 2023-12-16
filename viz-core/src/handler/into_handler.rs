use crate::{FromRequest, IntoResponse, Request, Result};

use super::{FnExt, FnExtHandler, Handler};

/// The trait implemented by types that can be converted to a [`Handler`].
pub trait IntoHandler<E, I> {
    /// The target handler.
    type Handler: Handler<I>;

    /// Converts self to a [`Handler`].
    #[must_use]
    fn into_handler(self) -> Self::Handler;
}

impl<H, E, O> IntoHandler<E, Request> for H
where
    E: FromRequest + 'static,
    E::Error: IntoResponse + Send,
    H: FnExt<E, Output = Result<O>>,
    O: 'static,
{
    type Handler = FnExtHandler<H, E, O>;

    fn into_handler(self) -> Self::Handler {
        FnExtHandler::new(self)
    }
}
