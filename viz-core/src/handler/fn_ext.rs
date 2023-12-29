use crate::{BoxFuture, Request};

/// A handler with extractors.
pub trait FnExt<E> {
    /// The returned type after the call operator is used.
    type Output;

    /// Performs the call operation.
    fn call(&self, req: Request) -> BoxFuture<Self::Output>;
}
