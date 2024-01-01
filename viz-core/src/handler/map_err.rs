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
