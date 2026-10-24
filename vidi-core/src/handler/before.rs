use crate::{Handler, Result};

impl_handler_combinator!(Before, "Maps the input before the handler calls.");

#[crate::async_trait]
impl<H, F, I, O> Handler<I> for Before<H, F>
where
    I: Send + 'static,
    F: Handler<I, Output = Result<I>>,
    H: Handler<I, Output = Result<O>>,
{
    type Output = H::Output;

    async fn call(&self, i: I) -> Self::Output {
        self.h.call(self.f.call(i).await?).await
    }
}
