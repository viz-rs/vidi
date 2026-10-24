use crate::{Handler, Result};

/// A middleware parameter: a tuple of input and handler.
pub type Next<I, H> = (I, H);

impl_handler_combinator!(
    Around,
    "Wraps around the remaining handler or middleware chain."
);

#[crate::async_trait]
impl<H, F, I, O> Handler<I> for Around<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>> + Clone,
    F: Handler<Next<I, H>, Output = H::Output>,
{
    type Output = F::Output;

    async fn call(&self, i: I) -> Self::Output {
        self.f.call((i, self.h.clone())).await
    }
}
