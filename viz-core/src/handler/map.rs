use crate::{async_trait, Handler, Result};

/// Maps the `Ok` value of the output if after the handler called.
#[derive(Debug, Clone)]
pub struct Map<H, F> {
    h: H,
    f: F,
}

impl<H, F> Map<H, F> {
    #[inline]
    pub(super) fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

#[async_trait]
impl<H, F, I, O> Handler<I> for Map<H, F>
where
    I: Send + 'static,
    O: Send,
    H: Handler<I, Output = Result<O>> + Clone,
    F: Handler<O, Output = O> + Clone,
{
    type Output = H::Output;

    async fn call(&self, i: I) -> Self::Output {
        Ok(self.f.call(self.h.call(i).await?).await)
    }
}
