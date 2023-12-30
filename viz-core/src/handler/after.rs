use crate::{future::FutureExt, BoxFuture, Handler, Result};

/// Maps the output `Result<T>` after the handler called.
#[derive(Debug, Clone)]
pub struct After<H, F> {
    h: H,
    f: F,
}

impl<H, F> After<H, F> {
    /// Creates an [`After`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

impl<H, F, I, O> Handler<I> for After<H, F>
where
    H: Handler<I, Output = Result<O>>,
    F: Handler<H::Output, Output = H::Output> + Send + Clone + 'static,
    O: 'static,
{
    type Output = F::Output;

    fn call(&self, i: I) -> BoxFuture<Self::Output> {
        let f = self.f.clone();
        Box::pin(self.h.call(i).then(move |o| f.call(o)))
    }
}
