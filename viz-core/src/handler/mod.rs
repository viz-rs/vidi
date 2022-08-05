//! Traits and types for handling an HTTP.

use crate::{async_trait, Future};

mod after;
mod and_then;
mod around;
mod before;
mod boxed;
mod catch_error;
mod catch_unwind;
mod either;
mod fn_ext;
mod fn_ext_hanlder;
mod into_handler;
mod map;
mod map_err;
mod map_into_response;
mod or_else;
mod transform;

pub use after::After;
pub use and_then::AndThen;
pub use around::{Around, Next};
pub use before::Before;
pub use boxed::BoxHandler;
pub use catch_error::CatchError;
pub use catch_unwind::CatchUnwind;
pub use either::Either;
pub use fn_ext::FnExt;
pub use fn_ext_hanlder::FnExtHandler;
pub use into_handler::IntoHandler;
pub use map::Map;
pub use map_err::MapErr;
pub use map_into_response::MapInToResponse;
pub use or_else::OrElse;
pub use transform::Transform;

/// A simplified asynchronous interface for handling input and output.
///
/// Composable request handlers.
#[async_trait]
pub trait Handler<Input>: dyn_clone::DynClone + Send + Sync + 'static {
    /// The returned type after the call operator is used.
    type Output;

    /// Performs the call operation.
    #[must_use]
    async fn call(&self, input: Input) -> Self::Output;
}

impl<I, T> HandlerExt<I> for T where T: Handler<I> + ?Sized {}

#[async_trait]
impl<F, I, Fut, O> Handler<I> for F
where
    I: Send + 'static,
    F: Fn(I) -> Fut + ?Sized + Clone + Send + Sync + 'static,
    Fut: Future<Output = O> + Send,
{
    type Output = Fut::Output;

    async fn call(&self, i: I) -> Self::Output {
        (self)(i).await
    }
}

/// The [`HandlerExt`] trait, which provides adapters for chaining and composing handlers.
///
/// Likes the [`FutureExt`] and [`StreamExt`] trait.
///
/// [`FutureExt`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html
/// [`StreamExt`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html
pub trait HandlerExt<I>: Handler<I> {
    /// Converts this Handler into a [BoxHandler].
    fn boxed(self) -> BoxHandler<I, Self::Output>
    where
        Self: Sized,
    {
        // box_into_inner
        Box::new(self)
    }

    /// Maps the input before the handler calls.
    fn before<F>(self, f: F) -> Before<Self, F>
    where
        Self: Sized,
    {
        Before::new(self, f)
    }

    /// Maps the output `Result<T>` after the handler called.
    fn after<F>(self, f: F) -> After<Self, F>
    where
        Self: Sized,
    {
        After::new(self, f)
    }

    /// Wraps around the remaining handler or middleware chain.
    fn around<F>(self, f: F) -> Around<Self, F>
    where
        Self: Sized,
    {
        Around::new(self, f)
    }

    /// Maps the `Ok` value of the output if after the handler called.
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map::new(self, f)
    }

    /// Maps the handler's output type to the [`Response`][crate::Response].
    fn map_into_response<O>(self) -> MapInToResponse<Self, O>
    where
        Self: Sized,
    {
        MapInToResponse::new(self)
    }

    /// Calls op if the result is Ok, otherwise returns the Err value of self.
    fn and_then<F>(self, f: F) -> AndThen<Self, F>
    where
        Self: Sized,
    {
        AndThen::new(self, f)
    }

    /// Maps the `Err` value of the output if after the handler called.
    fn map_err<F>(self, f: F) -> MapErr<Self, F>
    where
        Self: Sized,
    {
        MapErr::new(self, f)
    }

    /// Calls `op` if the output is `Err`, otherwise returns the `Ok` value of the output.
    fn or_else<F>(self, f: F) -> OrElse<Self, F>
    where
        Self: Sized,
    {
        OrElse::new(self, f)
    }

    /// Catches rejected error while calling the handler.
    fn catch_error<F, R, E>(self, f: F) -> CatchError<Self, F, R, E>
    where
        Self: Sized,
    {
        CatchError::new(self, f)
    }

    /// Catches unwinding panics while calling the handler.
    fn catch_unwind<F>(self, f: F) -> CatchUnwind<Self, F>
    where
        Self: Sized,
    {
        CatchUnwind::new(self, f)
    }

    /// Returns a new [Handler] that wrapping the `Self` and a type implementing [`Transform`].
    fn with<T>(self, t: T) -> T::Output
    where
        T: Transform<Self>,
        Self: Sized,
    {
        t.transform(self)
    }

    /// Maps the handler.
    fn with_fn<F>(self, f: F) -> Self
    where
        F: Fn(Self) -> Self,
        Self: Sized,
    {
        f(self)
    }
}
