use crate::Handler;

/// Combines two different handlers having the same associated types into a single type.
#[derive(Clone, Debug)]
pub enum Either<L, R> {
    /// First branch of the type.
    Left(L),
    /// Second branch of the type.
    Right(R),
}

#[crate::async_trait]
impl<L, R, I, O> Handler<I> for Either<L, R>
where
    I: Send + 'static,
    L: Handler<I, Output = O>,
    R: Handler<I, Output = O>,
{
    type Output = O;

    async fn call(&self, i: I) -> Self::Output {
        match self {
            Self::Left(l) => l.call(i),
            Self::Right(r) => r.call(i),
        }
        .await
    }
}
