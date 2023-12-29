use crate::{
    future::{FutureExt, TryFutureExt},
    BoxFuture, Handler, IntoResponse, Response, Result,
};

/// Catches unwinding panics while calling the handler.
#[derive(Debug, Clone)]
pub struct CatchUnwind<H, F> {
    h: H,
    f: F,
}

impl<H, F> CatchUnwind<H, F> {
    /// Creates an [`CatchUnwind`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

impl<H, F, I, O, R> Handler<I> for CatchUnwind<H, F>
where
    H: Handler<I, Output = Result<O>> + 'static,
    O: IntoResponse + 'static,
    F: Handler<Box<dyn ::core::any::Any + Send>, Output = R> + Send + Clone + 'static,
    R: IntoResponse + 'static,
{
    type Output = Result<Response>;

    fn call(&self, i: I) -> BoxFuture<Self::Output> {
        let f = self.f.clone();
        let fut = ::core::panic::AssertUnwindSafe(self.h.call(i))
            .catch_unwind()
            .map_ok(IntoResponse::into_response)
            .or_else(move |e| f.call(e).map(IntoResponse::into_response).map(Result::Ok));
        Box::pin(fut)
    }
}
