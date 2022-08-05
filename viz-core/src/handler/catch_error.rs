use std::marker::PhantomData;

use crate::{async_trait, Handler, IntoResponse, Response, Result};

/// Catches rejected error while calling the handler.
#[derive(Debug)]
pub struct CatchError<H, F, R, E> {
    h: H,
    f: F,
    _maker: PhantomData<(R, E)>,
}

impl<H, F, R, E> Clone for CatchError<H, F, R, E>
where
    H: Clone,
    F: Clone,
{
    fn clone(&self) -> Self {
        Self {
            h: self.h.clone(),
            f: self.f.clone(),
            _maker: PhantomData,
        }
    }
}

impl<H, F, R, E> CatchError<H, F, R, E> {
    #[inline]
    pub(super) fn new(h: H, f: F) -> Self {
        Self {
            h,
            f,
            _maker: PhantomData,
        }
    }
}

#[async_trait]
impl<H, F, I, O, R, E> Handler<I> for CatchError<H, F, E, R>
where
    I: Send + 'static,
    O: IntoResponse + Send,
    H: Handler<I, Output = Result<O>> + Clone,
    F: Handler<E, Output = R> + Clone,
    R: IntoResponse + Send + Sync + 'static,
    E: std::error::Error + Send + Sync + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, i: I) -> Self::Output {
        match self.h.call(i).await {
            Ok(r) => Ok(r.into_response()),
            Err(e) if e.is::<E>() => Ok(self
                .f
                .call(e.downcast::<E>().unwrap())
                .await
                .into_response()),
            Err(e) => Err(e),
        }
    }
}
