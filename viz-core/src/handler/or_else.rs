use crate::{Error, Handler, Result};

/// Calls `op` if the output is `Err`, otherwise returns the `Ok` value of the output.
#[derive(Debug, Clone)]
pub struct OrElse<H, F> {
    h: H,
    f: F,
}

impl<H, F> OrElse<H, F> {
    /// Creates an [`OrElse`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

#[crate::async_trait]
impl<H, F, I, O> Handler<I> for OrElse<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>>,
    F: Handler<Error, Output = H::Output>,
    O: Send,
{
    type Output = F::Output;

    async fn call(&self, i: I) -> Self::Output {
        match self.h.call(i).await {
            Ok(o) => Ok(o),
            Err(e) => self.f.call(e).await,
        }
    }
}
