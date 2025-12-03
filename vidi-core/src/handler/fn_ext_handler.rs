use std::marker::PhantomData;

use crate::{FnExt, FromRequest, Handler, IntoResponse, Result};

/// A wrapper of the extractors handler.
#[derive(Debug)]
pub struct FnExtHandler<H, E, O>(H, PhantomData<fn(E) -> O>);

impl<H, E, O> Clone for FnExtHandler<H, E, O>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<H, E, O> FnExtHandler<H, E, O> {
    /// Creates a new `Handler` for the extractors.
    pub fn new(h: H) -> Self {
        Self(h, PhantomData)
    }
}

#[crate::async_trait]
impl<I, H, E, O> Handler<I> for FnExtHandler<H, E, O>
where
    I: Send + 'static,
    E: FromRequest + 'static,
    E::Error: IntoResponse,
    H: FnExt<I, E, Output = Result<O>>,
    O: 'static,
{
    type Output = H::Output;

    async fn call(&self, i: I) -> Self::Output {
        self.0.call(i).await.map_err(IntoResponse::into_error)
    }
}
