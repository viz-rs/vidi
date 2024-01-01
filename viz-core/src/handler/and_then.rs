use crate::{Handler, Result};

/// Calls `op` if the output is `Ok`, otherwise returns the `Err` value of the output.
#[derive(Debug, Clone)]
pub struct AndThen<H, F> {
    h: H,
    f: F,
}

impl<H, F> AndThen<H, F> {
    /// Creates an [`AndThen`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

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
