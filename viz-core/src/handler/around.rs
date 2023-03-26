use crate::{async_trait, Handler, Result};

/// Represents a middleware parameter, which is a tuple that includes Requset and `BoxHandler`.
pub type Next<I, H> = (I, H);

/// Wraps around the remaining handler or middleware chain.
#[derive(Debug, Clone)]
pub struct Around<H, F> {
    h: H,
    f: F,
}

impl<H, F> Around<H, F> {
    #[inline]
    pub(super) fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

#[async_trait]
impl<H, F, I, O> Handler<I> for Around<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>> + Clone,
    F: Handler<Next<I, H>, Output = H::Output> + Clone,
{
    type Output = F::Output;

    async fn call(&self, i: I) -> Self::Output {
        self.f.call((i, self.h.clone())).await
    }
}
