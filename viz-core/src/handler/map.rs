use crate::{future::TryFutureExt, BoxFuture, Handler, Result};

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

impl<H, F, I, O, T> Handler<I> for Map<H, F>
where
    H: Handler<I, Output = Result<O>>,
    F: FnOnce(O) -> T + Send + Clone + 'static,
    O: 'static,
{
    type Output = Result<T>;

    fn call(&self, i: I) -> BoxFuture<Self::Output> {
        Box::pin(self.h.call(i).map_ok(self.f.clone()))
    }
}
