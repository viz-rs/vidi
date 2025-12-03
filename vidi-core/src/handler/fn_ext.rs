/// A handler with extractors.
pub trait FnExt<I, E>: Send + Sync + 'static {
    /// The returned type after the call operator is used.
    type Output;

    /// Performs the call operation.
    fn call(&self, i: I) -> impl crate::Future<Output = Self::Output> + Send;
}
