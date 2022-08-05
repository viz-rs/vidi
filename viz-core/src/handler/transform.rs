/// Then `Transform` trait defines the interface of a handler factory that wraps inner handler to
/// a Handler during construction.
pub trait Transform<H> {
    /// A new handler.
    type Output;

    /// Transforms `self` and wraps [Handler][super::Handler] to a new handler.
    fn transform(&self, h: H) -> Self::Output;
}
