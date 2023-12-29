use crate::{BoxFuture, Handler};

/// Combines two different handlers having the same associated types into a single type.
#[derive(Debug, Clone)]
pub enum Either<L, R> {
    /// First branch of the type.
    Left(L),
    /// Second branch of the type.
    Right(R),
}

impl<L, R, I, O> Handler<I> for Either<L, R>
where
    I: 'static,
    O: 'static,
    L: Handler<I, Output = O>,
    R: Handler<I, Output = O>,
{
    type Output = O;

    fn call(&self, i: I) -> BoxFuture<Self::Output> {
        Box::pin(match self {
            Self::Left(l) => l.call(i),
            Self::Right(r) => r.call(i),
        })
    }
}
