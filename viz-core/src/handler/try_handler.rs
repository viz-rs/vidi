use crate::{BoxFuture, Handler};

pub trait TryHandler<Input>: Handler<Input> {
    type Ok;

    type Error;

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

pub trait TryHandlerExt<I>: TryHandler<I> {}
