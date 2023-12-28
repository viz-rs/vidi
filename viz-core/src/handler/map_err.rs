use crate::{
    future::{BoxFuture, TryFutureExt},
    Error, Handler, Result,
};

/// Maps the `Err` value of the output if after the handler called.
#[derive(Debug, Clone)]
pub struct MapErr<H, F> {
    h: H,
    f: F,
}

impl<H, F> MapErr<H, F> {
    /// Creates a [`MapErr`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

impl<H, F, I, O, E> Handler<I> for MapErr<H, F>
where
    H: Handler<I, Output = Result<O, E>>,
    F: FnOnce(E) -> Error + Send + Clone + 'static,
    O: 'static,
    E: 'static,
{
    type Output = Result<O>;

    fn call(&self, i: I) -> BoxFuture<'static, Self::Output> {
        let fut = self.h.call(i).map_err(self.f.clone());
        Box::pin(fut)
    }
}
