use crate::{future::BoxFuture, Handler, Result};

/// Represents a middleware parameter, which is a tuple that includes Requset and `BoxHandler`.
pub type Next<I, H> = (I, H);

/// Wraps around the remaining handler or middleware chain.
#[derive(Debug, Clone)]
pub struct Around<H, F> {
    h: H,
    f: F,
}

impl<H, F> Around<H, F> {
    /// Creates an [`Around`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

impl<H, F, I, O> Handler<I> for Around<H, F>
where
    H: Handler<I, Output = Result<O>> + Clone + 'static,
    F: Handler<Next<I, H>, Output = H::Output>,
    O: 'static,
{
    type Output = F::Output;

    fn call(&self, i: I) -> BoxFuture<'static, Self::Output> {
        let h = self.h.clone();
        Box::pin(self.f.call((i, h)))
    }
}
