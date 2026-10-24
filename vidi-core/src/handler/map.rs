use crate::{Handler, Result};

impl_handler_combinator!(Map, "Maps the `Ok` value of the handler's output.");

#[crate::async_trait]
impl<H, F, I, O, T> Handler<I> for Map<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>>,
    F: FnOnce(O) -> T + Send + Sync + Copy + 'static,
{
    type Output = Result<T>;

    async fn call(&self, i: I) -> Self::Output {
        self.h.call(i).await.map(self.f)
    }
}
