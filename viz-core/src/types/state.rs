//! Represents a shared-    data extractor or handler/middleware.

use std::{
    any::type_name,
    fmt,
    ops::{Deref, DerefMut},
};

use crate::{
    async_trait, handler::Transform, types::PayloadError, FromRequest, Handler, IntoResponse,
    Request, RequestExt, Response, Result,
};

/// Extracts State from the extensions of a request.
pub struct State<T: ?Sized>(pub T);

impl<T> State<T> {
    /// Create new `State` instance.
    #[inline]
    pub fn new(data: T) -> Self {
        Self(data)
    }

    /// Consumes the State, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> Clone for State<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> AsRef<T> for State<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for State<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> DerefMut for State<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Debug for State<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        T::fmt(self, f)
    }
}

#[async_trait]
impl<T> FromRequest for State<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = PayloadError;

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.state().map(Self).ok_or_else(error::<T>)
    }
}

impl<H, T> Transform<H> for State<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Output = State<(H, T)>;

    fn transform(&self, h: H) -> Self::Output {
        State((h, self.0.clone()))
    }
}

// TODO: Maybe should be a `before` handler
#[async_trait]
impl<H, O, T> Handler<Request> for State<(H, T)>
where
    O: IntoResponse,
    H: Handler<Request, Output = Result<O>> + Clone,
    T: Clone + Send + Sync + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, mut req: Request) -> Self::Output {
        let Self((h, t)) = self;
        req.extensions_mut().insert(t.clone());
        h.call(req).await.map(IntoResponse::into_response)
    }
}

fn error<T>() -> PayloadError {
    PayloadError::State(type_name::<T>())
}
