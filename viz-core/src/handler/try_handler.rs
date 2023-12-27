use futures_util::future::BoxFuture;

use super::{Handler, MapErr};

pub trait TryHandler<Input>: Handler<Input> {
    type Ok;

    type Error;

    fn try_call(&self, input: Input) -> BoxFuture<'static, Result<Self::Ok, Self::Error>>;
}

impl<F, I, O, E> TryHandler<I> for F
where
    F: ?Sized + Handler<I, Output = Result<O, E>>,
{
    type Ok = O;
    type Error = E;

    #[inline]
    fn try_call(&self, input: I) -> BoxFuture<'static, Result<Self::Ok, Self::Error>> {
        self.call(input)
    }
}

pub trait TryHandlerExt<I>: TryHandler<I> {
    fn map_err<F, E>(self, f: F) -> MapErr<Self, F>
    where
        F: FnOnce(Self::Error) -> E,
        Self: Sized,
    {
        MapErr::new(self, f)
    }
}
