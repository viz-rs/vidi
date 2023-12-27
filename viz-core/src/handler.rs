//! Traits and types for handling an HTTP.

use crate::Future;
use futures_util::future::BoxFuture;

mod after;
pub use after::After;

mod and_then;
pub use and_then::AndThen;

mod around;
pub use around::{Around, Next};

mod before;
pub use before::Before;

mod boxed;
pub use boxed::BoxHandler;

mod catch_error;
pub use catch_error::CatchError;

mod catch_unwind;
pub use catch_unwind::CatchUnwind;

mod either;
pub use either::Either;

mod fn_ext;
pub use fn_ext::FnExt;

mod fn_ext_hanlder;
pub use fn_ext_hanlder::FnExtHandler;

mod into_handler;
pub use into_handler::IntoHandler;

mod map;
pub use map::Map;

mod map_err;
pub use map_err::MapErr;

mod map_into_response;
pub use map_into_response::MapInToResponse;

mod or_else;
pub use or_else::OrElse;

mod try_handler;
pub use try_handler::TryHandler;

mod transform;
pub use transform::Transform;

mod service;
pub use service::ServiceHandler;

/// A simplified asynchronous interface for handling input and output.
///
/// Composable request handlers.
pub trait Handler<Input> {
    /// The returned type after the call operator is used.
    type Output;

    /// Performs the call operation.
    fn call(&self, input: Input) -> BoxFuture<'static, Self::Output>;
}

impl<F, I, Fut, O> Handler<I> for F
where
    I: Send + 'static,
    F: Fn(I) -> Fut + ?Sized + Clone + Send + Sync + 'static,
    Fut: Future<Output = O> + Send,
{
    type Output = Fut::Output;

    fn call(&self, i: I) -> BoxFuture<'static, Self::Output> {
        Box::pin((self)(i))
    }
}

/// The [`HandlerExt`] trait, which provides adapters for chaining and composing handlers.
///
/// Likes the [`FutureExt`] and [`StreamExt`] traits.
///
/// [`FutureExt`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html
/// [`StreamExt`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html
pub trait HandlerExt<I>: Handler<I> {
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

    /// Wraps this handler in an Either handler, making it the left-hand variant of that Either.
    ///
    /// Returns the left-hand variant if `enable` is true, otherwise returns the right-hand
    /// variant.
    fn either<R>(self, r: R, enable: bool) -> Either<Self, R>
    where
        Self: Sized,
    {
        if enable {
            Either::Left(self)
        } else {
            Either::Right(r)
        }
    }

    /// Maps the `Ok` value of the output if after the handler called.
    fn map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map::new(self, f)
    }

    /// Maps the handler's output type to the [`Response`].
    ///
    /// [`Response`]: crate::Response
    fn map_into_response(self) -> MapInToResponse<Self>
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

    /// Converts this Handler into a [`BoxHandler`].
    fn boxed(self) -> BoxHandler<I, Self::Output>
    where
        Self: Sized,
    {
        Box::new(self)
    }

    /// Returns a new [`Handler`] that wrapping the `Self` and a type implementing [`Transform`].
    fn with<T>(self, t: T) -> T::Output
    where
        T: Transform<Self>,
        Self: Sized,
    {
        t.transform(self)
    }

    /// Maps the handler.
    #[must_use]
    fn with_fn<F>(self, f: F) -> Self
    where
        F: Fn(Self) -> Self,
        Self: Sized,
    {
        f(self)
    }
}

impl<I, T: ?Sized> HandlerExt<I> for T where T: Handler<I> {}
