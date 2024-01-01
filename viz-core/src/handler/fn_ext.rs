/// A handler with extractors.
#[crate::async_trait]
pub trait FnExt<I, E>: Send + Sync + 'static {
    /// The returned type after the call operator is used.
    type Output;

    /// Performs the call operation.
    async fn call(&self, i: I) -> Self::Output;
}
