use futures_util::{future::BoxFuture, TryFutureExt};

use crate::{Handler, Result};

/// Maps the input before the handler calls.
#[derive(Debug, Clone)]
pub struct Before<H, F> {
    h: H,
    f: F,
}

impl<H, F> Before<H, F> {
    /// Creates a [`Before`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

impl<H, F, I, O> Handler<I> for Before<H, F>
where
    I: Send + 'static,
    F: Handler<I, Output = Result<I>>,
    H: Handler<I, Output = Result<O>> + Send + Copy,
{
    type Output = H::Output;

    fn call(&self, i: I) -> BoxFuture<'static, Self::Output> {
        let h = self.h;
        let fut = self.f.call(i).and_then(move |i| h.call(i));
        Box::pin(fut)
    }
}
