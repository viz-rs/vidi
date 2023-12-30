use crate::{future::TryFutureExt, BoxFuture, Handler, Result};

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
    F: Handler<I, Output = Result<I>> + 'static,
    H: Handler<I, Output = Result<O>> + Send + Clone + 'static,
    O: 'static,
{
    type Output = H::Output;

    fn call(&self, i: I) -> BoxFuture<Self::Output> {
        let h = self.h.clone();
        Box::pin(self.f.call(i).and_then(move |i| h.call(i)))
    }
}
