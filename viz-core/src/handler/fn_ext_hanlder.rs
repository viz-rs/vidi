use std::marker::PhantomData;

use crate::{async_trait, FnExt, FromRequest, Handler, IntoResponse, Request, Result};

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

#[async_trait]
impl<H, E, O> Handler<Request> for FnExtHandler<H, E, O>
where
    E: FromRequest + 'static,
    E::Error: IntoResponse + Send,
    H: FnExt<E, Output = Result<O>>,
    O: 'static,
{
    type Output = H::Output;

    async fn call(&self, req: Request) -> Self::Output {
        self.0.call(req).await.map_err(IntoResponse::into_error)
    }
}
