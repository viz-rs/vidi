use crate::{async_trait, Error, Handler, Result};

/// Maps the `Err` value of the output if after the handler called.
#[derive(Debug, Clone)]
pub struct MapErr<H, F> {
    h: H,
    f: F,
}

impl<H, F> MapErr<H, F> {
    /// Creates a [`MapErr`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self { h, f }
    }
}

#[async_trait]
impl<H, F, I, O> Handler<I> for MapErr<H, F>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>> + Clone,
    O: Send,
    F: Handler<Error, Output = Error> + Clone,
{
    type Output = H::Output;

    async fn call(&self, i: I) -> Self::Output {
        match self.h.call(i).await {
            Ok(o) => Ok(o),
            Err(e) => Err(self.f.call(e).await),
        }
    }
}
