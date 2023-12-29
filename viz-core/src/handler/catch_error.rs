use std::marker::PhantomData;

use crate::{future::TryFutureExt, BoxFuture, Error, Handler, IntoResponse, Response, Result};

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

impl<H, I, O, F, E, R> Handler<I> for CatchError<H, F, E, R>
where
    H: Handler<I, Output = Result<O>>,
    O: IntoResponse + 'static,
    E: std::error::Error + Send + 'static,
    F: Handler<E, Output = R> + Send + Clone + 'static,
    R: IntoResponse,
{
    type Output = Result<Response>;

    fn call(&self, i: I) -> BoxFuture<Self::Output> {
        let f = self.f.clone();
        let fut = self
            .h
            .call(i)
            .map_ok(IntoResponse::into_response)
            .map_err(Error::downcast::<E>)
            .or_else(move |r| async move { Ok(f.call(r?).await.into_response()) });
        Box::pin(fut)
    }
}
