use crate::{async_trait, Handler, Result};

/// Maps the output `Result<T>` after the handler called.
#[derive(Debug, Clone)]
pub struct After<H, F> {
    h: H,
    f: F,
}

impl<H, F> After<H, F> {
    /// Creates an [`After`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

#[async_trait]
impl<H, F, I, O> Handler<I> for After<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>> + Clone,
    F: Handler<Result<O>, Output = Result<O>> + Clone,
{
    type Output = F::Output;

    async fn call(&self, i: I) -> Self::Output {
        self.f.call(self.h.call(i).await).await
    }
}
