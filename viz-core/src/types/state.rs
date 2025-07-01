//! Represents a shared-state extractor or handler/middleware.

use std::{
    any::type_name,
    ops::{Deref, DerefMut},
};

use crate::{
    Error, FromRequest, Handler, IntoResponse, Request, RequestExt, Response, Result, StatusCode,
    ThisError, handler::Transform,
};

/// Extracts state from the extensions of a request.
#[derive(Clone, Copy, Debug, Default)]
pub struct State<T: ?Sized>(pub T);

impl<T> State<T> {
    /// Create new `State` instance.
    #[must_use]
    #[inline]
    pub const fn new(data: T) -> Self {
        Self(data)
    }

    /// Consumes the State, returning the wrapped value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
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

impl<T> FromRequest for State<T>
where
    T: Clone + Send + Sync + 'static,
{
    type Error = StateError;

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

/// A [`State`] error.
#[derive(Debug, ThisError)]
#[error("missing state type `{0}`")]
pub struct StateError(pub &'static str);

impl StateError {
    /// Creates a `State` Error
    #[must_use]
    pub fn new<T>() -> Self {
        Self(type_name::<T>())
    }
}

impl IntoResponse for StateError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

impl From<StateError> for Error {
    fn from(e: StateError) -> Self {
        e.into_error()
    }
}
