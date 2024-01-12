//! A TLS listener wrapper.

/// Unified TLS listener type.
#[derive(Debug)]
pub struct TlsListener<T, A> {
    pub(crate) inner: T,
    pub(crate) acceptor: A,
}

impl<T, A> TlsListener<T, A> {
    /// Creates a new TLS listener.
    pub fn new(t: T, a: A) -> Self {
        Self {
            inner: t,
            acceptor: a,
        }
    }

    /// Gets the listener.
    pub fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets the acceptor.
    pub fn get_acceptor(&self) -> &A {
        &self.acceptor
    }
}
