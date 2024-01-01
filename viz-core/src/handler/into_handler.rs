use crate::{handler::FnExtHandler, FnExt, FromRequest, Handler, IntoResponse, Result};

/// The trait implemented by types that can be converted to a [`Handler`].
pub trait IntoHandler<I, E> {
    /// The target handler.
    type Handler: Handler<I>;

    /// Converts self to a [`Handler`].
    #[must_use]
    fn into_handler(self) -> Self::Handler;
}

impl<I, H, E, O> IntoHandler<I, E> for H
where
    I: Send + 'static,
    E: FromRequest + 'static,
    E::Error: IntoResponse,
    H: FnExt<I, E, Output = Result<O>>,
    O: 'static,
{
    type Handler = FnExtHandler<H, E, O>;

    fn into_handler(self) -> Self::Handler {
        FnExtHandler::new(self)
    }
}
