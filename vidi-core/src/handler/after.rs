use crate::{Handler, Result};

impl_handler_combinator!(
    After,
    "Maps the output `Result<T>` after the handler is called."
);

#[crate::async_trait]
impl<H, F, I, O> Handler<I> for After<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>>,
    F: Handler<H::Output, Output = H::Output>,
{
    type Output = F::Output;

    async fn call(&self, i: I) -> Self::Output {
        self.f.call(self.h.call(i).await).await
    }
}
