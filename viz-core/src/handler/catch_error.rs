use std::marker::PhantomData;

use futures_util::future::BoxFuture;

use crate::{Handler, IntoResponse, Response, Result};

/// Catches rejected error while calling the handler.
#[derive(Debug)]
pub struct CatchError<H, F, R, E> {
    h: H,
    f: F,
    _marker: PhantomData<fn(E) -> R>,
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
            _marker: PhantomData,
        }
    }
}

impl<H, F, R, E> CatchError<H, F, R, E> {
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

impl<H, F, I, O, R, E> Handler<I> for CatchError<H, F, R, E>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>> + Clone,
    O: IntoResponse + Send,
    E: std::error::Error + Send + 'static,
    F: Handler<E, Output = R> + Clone,
    R: IntoResponse + 'static,
{
    type Output = Result<Response>;

    fn call(&self, i: I) -> BoxFuture<'static, Self::Output> {
        match self.h.call(i).await {
            Ok(r) => Ok(r.into_response()),
            Err(e) => Ok(self.f.call(e.downcast::<E>()?).await.into_response()),
        }
    }
}
