use std::fmt;

use super::cloneable::BoxCloneable;
use crate::{Handler, Request, Response, Result};

/// A [`Clone`] + [`Send`] boxed [`Handler`].
pub struct BoxHandler<I = Request, O = Result<Response>>(BoxCloneable<I, O>);

impl<I, O> BoxHandler<I, O> {
    /// Creates a new `BoxHandler`.
    pub fn new<H>(h: H) -> Self
    where
        H: Handler<I, Output = O> + Send + Sync + Clone + 'static,
    {
        Self(Box::new(h))
    }
}

impl<I, O> Clone for BoxHandler<I, O>
where
    I: Send + 'static,
    O: 'static,
{
    fn clone(&self) -> Self {
        Self(self.0.clone_box())
    }
}

#[crate::async_trait]
impl<I, O> Handler<I> for BoxHandler<I, O>
where
    I: Send + 'static,
    O: 'static,
{
    type Output = O;

    async fn call(&self, i: I) -> Self::Output {
        self.0.call(i).await
    }
}

impl<I, O> From<BoxCloneable<I, O>> for BoxHandler<I, O> {
    fn from(value: BoxCloneable<I, O>) -> Self {
        Self(value)
    }
}

impl<I, O> fmt::Debug for BoxHandler<I, O> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BoxHandler").finish()
    }
}
