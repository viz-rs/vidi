use crate::{Handler, Result};

impl_handler_combinator!(AndThen, "Calls `op` if the output is `Ok`, otherwise returns the `Err` value of the output.");

#[crate::async_trait]
impl<H, F, I, O> Handler<I> for AndThen<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>>,
    F: Handler<O, Output = H::Output>,
    O: Send,
{
    type Output = F::Output;

    async fn call(&self, i: I) -> Self::Output {
        self.f.call(self.h.call(i).await?).await
    }
}
