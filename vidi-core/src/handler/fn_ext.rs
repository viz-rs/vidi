/// A handler with extractors.
pub trait FnExt<I, E>: Send + Sync + 'static {
    /// The output type.
    type Output;

    /// Calls the handler with the input.
    fn call(&self, i: I) -> impl crate::Future<Output = Self::Output> + Send;
}
