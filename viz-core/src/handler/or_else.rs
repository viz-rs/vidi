use crate::{
    future::{BoxFuture, TryFutureExt},
    Error, Handler, Result,
};

/// Calls `op` if the output is `Err`, otherwise returns the `Ok` value of the output.
#[derive(Debug, Clone)]
pub struct OrElse<H, F> {
    h: H,
    f: F,
}

impl<H, F> OrElse<H, F> {
    /// Creates an [`OrElse`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

impl<H, F, I, O> Handler<I> for OrElse<H, F>
where
    H: Handler<I, Output = Result<O>>,
    F: Handler<Error, Output = H::Output> + Send + Clone + 'static,
    O: 'static,
{
    type Output = F::Output;

    fn call(&self, i: I) -> BoxFuture<'static, Self::Output> {
        let f = self.f.clone();
        let fut = self.h.call(i).or_else(move |e| f.call(e));
        Box::pin(fut)
    }
}
