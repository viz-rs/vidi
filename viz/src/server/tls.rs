//! A TLS listener wrapper.

/// `native_tls`
#[cfg(feature = "native-tls")]
pub mod native_tls;

/// `rustls`
#[cfg(feature = "rustls")]
pub mod rustls;

/// Unified TLS listener type.
#[derive(Debug)]
pub struct TlsListener<T, A> {
    pub(crate) inner: T,
    pub(crate) acceptor: A,
}

impl<T, A> TlsListener<T, A> {
    /// Creates a new TLS listener.
    pub const fn new(t: T, a: A) -> Self {
        Self {
            inner: t,
            acceptor: a,
        }
    }

    /// Gets the listener.
    pub const fn get_ref(&self) -> &T {
        &self.inner
    }

    /// Gets the acceptor.
    pub const fn get_acceptor(&self) -> &A {
        &self.acceptor
    }
}
