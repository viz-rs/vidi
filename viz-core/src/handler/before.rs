use crate::{async_trait, Handler, Result};

/// Maps the input before the handler calls.
#[derive(Debug, Clone)]
pub struct Before<H, F> {
    h: H,
    f: F,
}

impl<H, F> Before<H, F> {
    #[inline]
    pub(super) fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

#[async_trait]
impl<H, F, I, O> Handler<I> for Before<H, F>
where
    I: Send + 'static,
    F: Handler<I, Output = Result<I>> + Clone,
    H: Handler<I, Output = Result<O>> + Clone,
{
    type Output = H::Output;

    async fn call(&self, i: I) -> Self::Output {
        self.h.call(self.f.call(i).await?).await
    }
}
