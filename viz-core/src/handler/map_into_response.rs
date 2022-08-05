use std::marker::PhantomData;

use crate::{async_trait, Handler, IntoResponse, Response, Result};

/// Maps the handler's output type to the [`Response`].
#[derive(Debug)]
pub struct MapInToResponse<H, O>(pub(crate) H, PhantomData<O>);

impl<H, O> MapInToResponse<H, O> {
    /// Creates a new `Responder`.
    pub(super) fn new(h: H) -> Self {
        Self(h, PhantomData)
    }
}

impl<H, O> Clone for MapInToResponse<H, O>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

#[async_trait]
impl<H, I, O> Handler<I> for MapInToResponse<H, O>
where
    I: Send + 'static,
    H: Handler<I, Output = Result<O>> + Clone,
    O: IntoResponse + Send + Sync + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, args: I) -> Self::Output {
        self.0.call(args).await.map(IntoResponse::into_response)
    }
}
