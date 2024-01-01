use std::{
    io::{Error as IoError, ErrorKind, Result as IoResult},
    net::SocketAddr,
    task::{Context, Poll},
};

use futures_util::FutureExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{
    rustls::{
        server::{
            AllowAnyAnonymousOrAuthenticatedClient, AllowAnyAuthenticatedClient, NoClientAuth,
        },
        Certificate, PrivateKey, RootCertStore, ServerConfig,
    },
    server::TlsStream,
};

use super::Listener;
use crate::{Error, Result};

pub use tokio_rustls::TlsAcceptor;

/// Tls client authentication configuration.
#[derive(Debug)]
pub(crate) enum ClientAuth {
    /// No client auth.
    Off,
    /// Allow any anonymous or authenticated client.
    Optional(Vec<u8>),
    /// Allow any authenticated client.
    Required(Vec<u8>),
}

/// `rustls`'s config.
#[derive(Debug)]
pub struct Config {
    cert: Vec<u8>,
    key: Vec<u8>,
    ocsp_resp: Vec<u8>,
    client_auth: ClientAuth,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Create a new Tls config
    #[must_use]
    pub fn new() -> Self {
        Self {
            cert: Vec::new(),
            key: Vec::new(),
            client_auth: ClientAuth::Off,
            ocsp_resp: Vec::new(),
        }
    }

    /// sets the Tls certificate
    #[must_use]
    pub fn cert(mut self, cert: impl Into<Vec<u8>>) -> Self {
        self.cert = cert.into();
        self
    }

    /// sets the Tls key
    #[must_use]
    pub fn key(mut self, key: impl Into<Vec<u8>>) -> Self {
        self.key = key.into();
        self
    }

    /// Sets the trust anchor for optional Tls client authentication
    #[must_use]
    pub fn client_auth_optional(mut self, trust_anchor: impl Into<Vec<u8>>) -> Self {
        self.client_auth = ClientAuth::Optional(trust_anchor.into());
        self
    }

    /// Sets the trust anchor for required Tls client authentication
    #[must_use]
    pub fn client_auth_required(mut self, trust_anchor: impl Into<Vec<u8>>) -> Self {
        self.client_auth = ClientAuth::Required(trust_anchor.into());
        self
    }

    /// sets the DER-encoded OCSP response
    #[must_use]
    pub fn ocsp_resp(mut self, ocsp_resp: impl Into<Vec<u8>>) -> Self {
        self.ocsp_resp = ocsp_resp.into();
        self
    }

    /// builds the Tls `ServerConfig`
    ///
    /// # Errors
    pub fn build(self) -> Result<ServerConfig> {
        fn read_trust_anchor(trust_anchor: &Certificate) -> Result<RootCertStore> {
            let mut store = RootCertStore::empty();
            store.add(trust_anchor).map_err(Error::boxed)?;
            Ok(store)
        }

        let certs = rustls_pemfile::certs(&mut self.cert.as_slice())
            .map(|mut certs| certs.drain(..).map(Certificate).collect())
            .map_err(Error::boxed)?;

        let keys = {
            let mut pkcs8: Vec<PrivateKey> =
                rustls_pemfile::pkcs8_private_keys(&mut self.key.as_slice())
                    .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
                    .map_err(Error::boxed)?;
            if pkcs8.is_empty() {
                let mut rsa: Vec<PrivateKey> =
                    rustls_pemfile::rsa_private_keys(&mut self.key.as_slice())
                        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
                        .map_err(Error::boxed)?;

                if rsa.is_empty() {
                    return Err(Error::boxed(IoError::new(
                        ErrorKind::InvalidData,
                        "failed to parse tls private keys",
                    )));
                }
                rsa.remove(0)
            } else {
                pkcs8.remove(0)
            }
        };

        let client_auth = match self.client_auth {
            ClientAuth::Off => NoClientAuth::boxed(),
            ClientAuth::Optional(trust_anchor) => AllowAnyAnonymousOrAuthenticatedClient::new(
                read_trust_anchor(&Certificate(trust_anchor))?,
            )
            .boxed(),
            ClientAuth::Required(trust_anchor) => {
                AllowAnyAuthenticatedClient::new(read_trust_anchor(&Certificate(trust_anchor))?)
                    .boxed()
            }
        };

        ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(client_auth)
            .with_single_cert_with_ocsp_and_sct(certs, keys, self.ocsp_resp, Vec::new())
            .map_err(Error::boxed)
    }
}

impl tokio_util::net::Listener for Listener<TcpListener, TlsAcceptor> {
    type Io = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    fn poll_accept(&mut self, cx: &mut Context<'_>) -> Poll<IoResult<(Self::Io, Self::Addr)>> {
        let Poll::Ready((stream, addr)) = self.inner.poll_accept(cx)? else {
            return Poll::Pending;
        };
        self.acceptor
            .accept(stream)
            .poll_unpin(cx)
            .map_ok(|stream| (stream, addr))
    }

    fn local_addr(&self) -> IoResult<Self::Addr> {
        self.inner.local_addr()
    }
}
