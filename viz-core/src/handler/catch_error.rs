use std::marker::PhantomData;

use crate::{async_trait, Handler, IntoResponse, Response, Result};

/// Catches rejected error while calling the handler.
#[derive(Debug)]
pub struct CatchError<H, F, E, R> {
    h: H,
    f: F,
    _marker: PhantomData<fn(E) -> R>,
}

impl<H, F, E, R> Clone for CatchError<H, F, E, R>
where
    H: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            h: self.h.clone(),
            f: self.f.clone(),
            _marker: PhantomData,
        }
    }
}

impl<H, F, E, R> CatchError<H, F, E, R> {
    /// Creates a [`CatchError`] handler.
    #[inline]
    pub fn new(h: H, f: F) -> Self {
        Self {
            h,
            f,
            _marker: PhantomData,
        }
    }
}

#[async_trait]
impl<H, I, O, F, E, R> Handler<I> for CatchError<H, F, E, R>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>>,
    O: IntoResponse + Send,
    E: std::error::Error + Send + 'static,
    F: Handler<E, Output = R>,
    R: IntoResponse + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, i: I) -> Self::Output {
        match self.h.call(i).await {
            Ok(r) => Ok(r.into_response()),
            Err(e) => Ok(self.f.call(e.downcast::<E>()?).await.into_response()),
        }
    }
}
