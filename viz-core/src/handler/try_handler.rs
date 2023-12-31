use crate::{BoxFuture, Handler};

/// A convenience for handlers that return `Result` values that includes
/// a variety of adapters tailored to such futures.
pub trait TryHandler<Input>: Handler<Input> {
    /// The type of successful values yielded by this handler
    type Ok;

    /// The type of failures yielded by this handler
    type Error;

    /// Call this `TryHandler` as if it were a `Handler`.
    ///
    /// This method is a stopgap for a compiler limitation that prevents us from
    /// directly inheriting from the `Handler` trait; in the future it won't be
    /// needed.
    fn try_call(&self, input: Input) -> BoxFuture<Result<Self::Ok, Self::Error>>;
}

impl<F, I, O, E> TryHandler<I> for F
where
    F: ?Sized + Handler<I, Output = Result<O, E>>,
{
    type Ok = O;
    type Error = E;

    #[inline]
    fn try_call(&self, input: I) -> BoxFuture<Result<Self::Ok, Self::Error>> {
        self.call(input)
    }
}

pub(crate) trait TryHandlerExt<I>: TryHandler<I> {}
