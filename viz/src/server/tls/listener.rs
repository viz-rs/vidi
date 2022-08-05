use std::marker::PhantomData;

/// Unified TLS listener type.
#[derive(Debug)]
pub struct Listener<T, A, IO> {
    pub(crate) inner: T,
    pub(crate) acceptor: A,
    _marker: PhantomData<IO>,
}

impl<T, A, IO> Listener<T, A, IO> {
    /// Creates a new TLS listener.
    pub fn new(t: T, a: A) -> Self {
        Self {
            inner: t,
            acceptor: a,
            _marker: PhantomData,
        }
    }
}
