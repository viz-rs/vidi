/// The [`Transform`] trait defines the interface of a handler factory that wraps an inner
/// handler to produce a new handler during construction.
pub trait Transform<H> {
    /// A new handler.
    type Output;

    /// Transforms `self` and wraps [`Handler`][super::Handler] to a new handler.
    #[must_use]
    fn transform(&self, h: H) -> Self::Output;
}
