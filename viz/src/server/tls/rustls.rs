use std::{
    io::{Error as IoError, ErrorKind, Result as IoResult},
    net::SocketAddr,
};

use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{
    rustls::{pki_types::PrivateKeyDer, server::WebPkiClientVerifier, RootCertStore, ServerConfig},
    server::TlsStream,
};

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
        fn read_trust_anchor(mut trust_anchor: &[u8]) -> Result<RootCertStore> {
            let certs = rustls_pemfile::certs(&mut trust_anchor)
                .collect::<IoResult<Vec<_>>>()
                .map_err(Error::boxed)?;
            let mut store = RootCertStore::empty();
            for cert in certs {
                store.add(cert).map_err(Error::boxed)?;
            }
            Ok(store)
        }

        let certs = rustls_pemfile::certs(&mut self.cert.as_slice())
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::boxed)?;

        let keys = {
            let mut pkcs8 = rustls_pemfile::pkcs8_private_keys(&mut self.key.as_slice())
                .collect::<Result<Vec<_>, _>>()
                .map_err(Error::boxed)?;
            if pkcs8.is_empty() {
                let mut rsa = rustls_pemfile::rsa_private_keys(&mut self.key.as_slice())
                    .collect::<Result<Vec<_>, _>>()
                    .map_err(Error::boxed)?;

                if rsa.is_empty() {
                    return Err(Error::boxed(IoError::new(
                        ErrorKind::InvalidData,
                        "failed to parse tls private keys",
                    )));
                }
                PrivateKeyDer::Pkcs1(rsa.remove(0))
            } else {
                PrivateKeyDer::Pkcs8(pkcs8.remove(0))
            }
        };

        let client_auth = match self.client_auth {
            ClientAuth::Off => WebPkiClientVerifier::no_client_auth(),
            ClientAuth::Optional(trust_anchor) => {
                WebPkiClientVerifier::builder(read_trust_anchor(&trust_anchor)?.into())
                    .allow_unauthenticated()
                    .build()
                    .map_err(Error::boxed)?
            }
            ClientAuth::Required(trust_anchor) => {
                WebPkiClientVerifier::builder(read_trust_anchor(&trust_anchor)?.into())
                    .build()
                    .map_err(Error::boxed)?
            }
        };

        ServerConfig::builder()
            .with_client_cert_verifier(client_auth)
            .with_single_cert_with_ocsp(certs, keys, self.ocsp_resp)
            .map_err(Error::boxed)
    }
}

impl crate::Listener for crate::tls::TlsListener<TcpListener, TlsAcceptor> {
    type Io = TlsStream<TcpStream>;
    type Addr = SocketAddr;

    async fn accept(&self) -> IoResult<(Self::Io, Self::Addr)> {
        let (stream, addr) = self.inner.accept().await?;
        let stream = self.acceptor.accept(stream).await?;
        Ok((stream, addr))
    }

    fn local_addr(&self) -> IoResult<Self::Addr> {
        self.inner.local_addr()
    }
}
