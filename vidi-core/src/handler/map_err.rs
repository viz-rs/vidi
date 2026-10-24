use crate::{Error, Handler, Result};

impl_handler_combinator!(MapErr, "Maps the `Err` value of the handler's output.");

#[crate::async_trait]
impl<H, F, I, O, E> Handler<I> for MapErr<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O, E>>,
    F: FnOnce(E) -> Error + Send + Sync + Copy + 'static,
{
    type Output = Result<O>;

    async fn call(&self, i: I) -> Self::Output {
        self.h.call(i).await.map_err(self.f)
    }
}
