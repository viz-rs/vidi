//! Shared-state extractor and middleware.

use std::{
    any::type_name,
    ops::{Deref, DerefMut},
};

use crate::{
    Error, FromRequest, Handler, IntoResponse, Request, RequestExt, Response, Result, StatusCode,
    ThisError, handler::Transform,
};

/// Extracts shared state from the request.
#[derive(Clone, Copy, Debug, Default)]
pub struct State<T: ?Sized>(pub T);

impl<T> State<T> {
    /// Creates a [`State`].
    #[must_use]
    #[inline]
    pub const fn new(data: T) -> Self {
        Self(data)
    }

    /// Returns the wrapped value, consuming the state.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl_deref!(State);

impl<T> FromRequest for State<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = StateError;

    #[allow(clippy::unused_async_trait_impl)]
    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.state().map(Self).ok_or_else(StateError::new::<T>)
    }
}

impl<H, T> Transform<H> for State<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Output = State<(T, H)>;

    fn transform(&self, h: H) -> Self::Output {
        State((self.0.clone(), h))
    }
}

#[crate::async_trait]
impl<T, H, O> Handler<Request> for State<(T, H)>
where
    T: Clone + Send + Sync + 'static,
    H: Handler<Request, Output = Result<O>>,
    O: IntoResponse,
{
    type Output = Result<Response>;

    async fn call(&self, mut req: Request) -> Self::Output {
        let Self((t, h)) = self;
        req.extensions_mut().insert(t.clone());
        h.call(req).await.map(IntoResponse::into_response)
    }
}

/// Error returned when state extraction fails.
#[derive(Debug, ThisError)]
#[error("missing state type `{0}`")]
pub struct StateError(pub &'static str);

impl StateError {
    /// Creates a [`StateError`] for type `T`.
    #[must_use]
    pub fn new<T>() -> Self {
        Self(type_name::<T>())
    }
}

impl_error!(StateError);
