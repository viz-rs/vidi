use crate::Middleware;

/// Transforms a Tower layer into Viz Middleware.
#[derive(Debug)]
pub struct Layered<L>(L);

impl<L> Layered<L> {
    /// Creates a new tower layer.
    pub fn new(l: L) -> Self {
        Self(l)
    }
}

impl<L, H> viz_core::Transform<H> for Layered<L>
where
    L: Clone,
{
    type Output = Middleware<L, H>;

    fn transform(&self, h: H) -> Self::Output {
        Middleware::new(self.0.clone(), h)
    }
}
