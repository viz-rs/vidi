use std::{
    convert::Infallible,
    future::{self, Ready},
    io::{Error as IoError, ErrorKind},
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use hyper::{
    server::{
        accept::Accept,
        conn::{AddrIncoming, AddrStream},
    },
    service::Service,
};
use tokio_rustls::{
    rustls::{
        server::{
            AllowAnyAnonymousOrAuthenticatedClient, AllowAnyAuthenticatedClient, NoClientAuth,
        },
        Certificate, PrivateKey, RootCertStore, ServerConfig,
    },
    server::TlsStream,
    Accept as TlsAccept,
};

use crate::{Error, Responder, Result, ServiceMaker};

use super::{Listener, Stream};

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
    pub fn new() -> Self {
        Self {
            cert: Vec::new(),
            key: Vec::new(),
            client_auth: ClientAuth::Off,
            ocsp_resp: Vec::new(),
        }
    }

    /// sets the Tls certificate
    pub fn cert(mut self, cert: impl Into<Vec<u8>>) -> Self {
        self.cert = cert.into();
        self
    }

    /// sets the Tls key
    pub fn key(mut self, key: impl Into<Vec<u8>>) -> Self {
        self.key = key.into();
        self
    }

    /// Sets the trust anchor for optional Tls client authentication
    pub fn client_auth_optional(mut self, trust_anchor: impl Into<Vec<u8>>) -> Self {
        self.client_auth = ClientAuth::Optional(trust_anchor.into());
        self
    }

    /// Sets the trust anchor for required Tls client authentication
    pub fn client_auth_required(mut self, trust_anchor: impl Into<Vec<u8>>) -> Self {
        self.client_auth = ClientAuth::Required(trust_anchor.into());
        self
    }

    /// sets the DER-encoded OCSP response
    pub fn ocsp_resp(mut self, ocsp_resp: impl Into<Vec<u8>>) -> Self {
        self.ocsp_resp = ocsp_resp.into();
        self
    }

    /// builds the Tls ServerConfig
    pub fn build(self) -> Result<ServerConfig> {
        let certs = rustls_pemfile::certs(&mut self.cert.as_slice())
            .map(|mut certs| certs.drain(..).map(Certificate).collect())
            .map_err(Error::normal)?;

        let keys = {
            let mut pkcs8: Vec<PrivateKey> =
                rustls_pemfile::pkcs8_private_keys(&mut self.key.as_slice())
                    .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
                    .map_err(Error::normal)?;
            if !pkcs8.is_empty() {
                pkcs8.remove(0)
            } else {
                let mut rsa: Vec<PrivateKey> =
                    rustls_pemfile::rsa_private_keys(&mut self.key.as_slice())
                        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
                        .map_err(Error::normal)?;

                if !rsa.is_empty() {
                    rsa.remove(0)
                } else {
                    return Err(Error::normal(IoError::new(
                        ErrorKind::InvalidData,
                        "failed to parse tls private keys",
                    )));
                }
            }
        };

        fn read_trust_anchor(trust_anchor: &Certificate) -> Result<RootCertStore> {
            let mut store = RootCertStore::empty();
            store.add(trust_anchor).map_err(Error::normal)?;
            Ok(store)
        }

        let client_auth = match self.client_auth {
            ClientAuth::Off => NoClientAuth::new(),
            ClientAuth::Optional(trust_anchor) => AllowAnyAnonymousOrAuthenticatedClient::new(
                read_trust_anchor(&Certificate(trust_anchor))?,
            ),
            ClientAuth::Required(trust_anchor) => {
                AllowAnyAuthenticatedClient::new(read_trust_anchor(&Certificate(trust_anchor))?)
            }
        };

        ServerConfig::builder()
            .with_safe_defaults()
            .with_client_cert_verifier(client_auth)
            .with_single_cert_with_ocsp_and_sct(certs, keys, self.ocsp_resp, Vec::new())
            .map_err(Error::normal)
    }
}

impl Accept for Listener<AddrIncoming, TlsAcceptor, AddrStream> {
    type Conn = Stream<TlsAccept<AddrStream>, TlsStream<AddrStream>>;
    type Error = IoError;

    fn poll_accept(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        match ready!(Pin::new(&mut self.inner).poll_accept(cx)) {
            Some(Ok(sock)) => Poll::Ready(Some(Ok({
                let remote_addr = sock.remote_addr();
                Stream::new(self.acceptor.accept(sock), Some(remote_addr))
            }))),
            Some(Err(e)) => Poll::Ready(Some(Err(e))),
            None => Poll::Ready(None),
        }
    }
}

impl Service<&Stream<TlsAccept<AddrStream>, TlsStream<AddrStream>>> for ServiceMaker {
    type Response = Responder;
    type Error = Infallible;
    type Future = Ready<Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, t: &Stream<TlsAccept<AddrStream>, TlsStream<AddrStream>>) -> Self::Future {
        future::ready(Ok(Responder::new(self.tree.clone(), t.remote_addr)))
    }
}
