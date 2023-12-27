use futures_util::{future::BoxFuture, TryFutureExt};

use crate::{Handler, Result};

/// Calls `op` if the output is `Ok`, otherwise returns the `Err` value of the output.
#[derive(Debug, Clone)]
pub struct AndThen<H, F> {
    h: H,
    f: F,
}

impl<H, F> AndThen<H, F> {
    /// Creates an [`AndThen`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

impl<H, F, I, O> Handler<I> for AndThen<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>>,
    O: Send,
    F: Handler<O, Output = H::Output> + Send + Copy,
{
    type Output = F::Output;

    fn call(&self, i: I) -> BoxFuture<'static, Self::Output> {
        let f = self.f;
        let fut = self.h.call(i).and_then(move |o| f.call(o));
        Box::pin(fut)
    }
}
