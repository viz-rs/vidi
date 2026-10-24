use crate::{Handler, IntoResponse, Response, Result, future::FutureExt};

impl_handler_combinator!(
    CatchUnwind,
    "Catches unwinding panics while calling the handler."
);

#[crate::async_trait]
impl<H, F, I, O, R> Handler<I> for CatchUnwind<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>>,
    O: IntoResponse + Send,
    F: Handler<Box<dyn ::core::any::Any + Send>, Output = R>,
    R: IntoResponse,
{
    type Output = Result<Response>;

    async fn call(&self, i: I) -> Self::Output {
        match ::core::panic::AssertUnwindSafe(self.h.call(i))
            .catch_unwind()
            .await
        {
            Ok(r) => r.map(IntoResponse::into_response),
            Err(e) => Ok(self.f.call(e).await.into_response()),
        }
    }
}
