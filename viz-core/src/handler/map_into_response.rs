use crate::{future::TryFutureExt, BoxFuture, Handler, IntoResponse, Response, Result};

/// Maps the handler's output type to the [`Response`].
#[derive(Debug, Clone)]
pub struct MapInToResponse<H>(pub(crate) H);

impl<H> MapInToResponse<H> {
    /// Creates a [`MapInToResponse`] handler.
    #[inline]
    pub fn new(h: H) -> Self {
        Self(h)
    }
}

impl<H, I, O> Handler<I> for MapInToResponse<H>
where
    H: Handler<I, Output = Result<O>>,
    O: IntoResponse + 'static,
{
    type Output = Result<Response>;

    fn call(&self, i: I) -> BoxFuture<Self::Output> {
        let fut = self.0.call(i).map_ok(IntoResponse::into_response);
        Box::pin(fut)
    }
}
