use futures_util::{future::BoxFuture, FutureExt, TryFutureExt};

use crate::{Handler, Result};

/// Maps the `Ok` value of the output if after the handler called.
#[derive(Debug, Clone)]
pub struct Map<H, F> {
    h: H,
    f: F,
}

impl<H, F> Map<H, F> {
    /// Creates a [`Map`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

impl<H, F, I, O> Handler<I> for Map<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>>,
    O: Send + 'static,
    F: Handler<O, Output = O> + Copy,
{
    type Output = H::Output;

    fn call(&self, i: I) -> BoxFuture<'static, Self::Output> {
        let f = self.f;
        let fut = self.h.call(i).map_ok(move |o| f.call(o));
        Box::pin(fut)
    }
}
