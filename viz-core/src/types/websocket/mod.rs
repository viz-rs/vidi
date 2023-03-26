//! `WebSocket` Extractor

use std::{borrow::Cow, future::Future};

use hyper::upgrade::{OnUpgrade, Upgraded};
use tokio_tungstenite::tungstenite::protocol::Role;

use crate::{
    async_trait,
    header::{SEC_WEBSOCKET_PROTOCOL, UPGRADE},
    headers::{
        Connection, HeaderMapExt, HeaderValue, SecWebsocketAccept, SecWebsocketKey,
        SecWebsocketVersion, Upgrade,
    },
    FromRequest, IntoResponse, OutgoingBody, Request, Response, Result, StatusCode,
};

mod error;

pub use error::WebSocketError;
pub use tokio_tungstenite::tungstenite::protocol::{Message, WebSocketConfig};

/// A wrapper around an underlying raw stream which implements the `WebSocket` protocol.
pub type WebSocketStream<T = Upgraded> = tokio_tungstenite::WebSocketStream<T>;

/// Then `WebSocket` provides the API for creating and managing a [`WebSocket`][mdn] connection,
/// as well as for sending and receiving data on the connection.
///
/// [mdn]: <https://developer.mozilla.org/en-US/docs/Web/API/WebSocket>
#[derive(Debug)]
pub struct WebSocket {
    key: SecWebsocketKey,
    on_upgrade: Option<OnUpgrade>,
    protocols: Option<Box<[Cow<'static, str>]>>,
    sec_websocket_protocol: Option<HeaderValue>,
}

impl WebSocket {
    const NAME: &'static [u8] = b"websocket";

    /// The specifies one or more protocols that you wish to use.
    ///
    /// In order of preference. The first one that is supported by the server will be
    /// selected and responsed.
    pub fn protocols<I>(mut self, protocols: I) -> Self
    where
        I: IntoIterator,
        I::Item: Into<Cow<'static, str>>,
    {
        self.protocols = Some(
            protocols
                .into_iter()
                .map(Into::into)
                .collect::<Vec<_>>()
                .into(),
        );
        self
    }

    /// Finish the upgrade, passing a function and a [`WebSocketConfig`] to handle the `WebSocket`.
    #[must_use]
    pub fn on_upgrade_with_config<F, Fut>(
        mut self,
        callback: F,
        config: Option<WebSocketConfig>,
    ) -> Response
    where
        F: FnOnce(WebSocketStream) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let on_upgrade = self.on_upgrade.take().unwrap();

        tokio::task::spawn(async move {
            let upgraded = match on_upgrade.await {
                Ok(upgraded) => upgraded,
                Err(_) => return,
            };

            let socket = WebSocketStream::from_raw_socket(upgraded, Role::Server, config).await;

            (callback)(socket).await
        });

        self.into_response()
    }

    /// Finish the upgrade, passing a function to handle the `WebSocket`.
    pub fn on_upgrade<F, Fut>(self, callback: F) -> Response
    where
        F: FnOnce(WebSocketStream) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.on_upgrade_with_config(callback, None)
    }
}

#[async_trait]
impl FromRequest for WebSocket {
    type Error = WebSocketError;

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        // check connection header
        req.headers()
            .typed_get::<Connection>()
            .ok_or(WebSocketError::MissingConnectUpgrade)
            .and_then(|h| {
                if h.contains(UPGRADE) {
                    Ok(())
                } else {
                    Err(WebSocketError::InvalidConnectUpgrade)
                }
            })?;

        // check upgrade header
        req.headers()
            .get(UPGRADE)
            .ok_or(WebSocketError::MissingUpgrade)
            .and_then(|h| {
                if h.as_bytes().eq_ignore_ascii_case(WebSocket::NAME) {
                    Ok(())
                } else {
                    Err(WebSocketError::InvalidUpgrade)
                }
            })?;

        // check sec-websocket-version header
        req.headers()
            .typed_get::<SecWebsocketVersion>()
            .ok_or(WebSocketError::MissingWebSocketVersion)
            .and_then(|h| {
                if h == SecWebsocketVersion::V13 {
                    Ok(())
                } else {
                    Err(WebSocketError::InvalidWebSocketVersion)
                }
            })?;

        let key = req
            .headers()
            .typed_get::<SecWebsocketKey>()
            .ok_or(WebSocketError::MissingWebSocketKey)?;

        let on_upgrade = req.extensions_mut().remove::<OnUpgrade>();

        if on_upgrade.is_none() {
            Err(WebSocketError::ConnectionNotUpgradable)?;
        }

        let sec_websocket_protocol = req.headers().get(SEC_WEBSOCKET_PROTOCOL).cloned();

        Ok(Self {
            key,
            on_upgrade,
            protocols: None,
            sec_websocket_protocol,
        })
    }
}

impl IntoResponse for WebSocket {
    fn into_response(self) -> Response {
        let protocol = self
            .sec_websocket_protocol
            .as_ref()
            .and_then(|req_protocols| {
                let req_protocols = req_protocols.to_str().ok()?;
                let protocols = self.protocols.as_ref()?;
                req_protocols
                    .split(',')
                    .map(|req_p| req_p.trim())
                    .find(|req_p| protocols.iter().any(|p| p == req_p))
                    .and_then(|v| HeaderValue::from_str(v).ok())
            });

        let mut res = Response::new(OutgoingBody::Empty);

        *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
        res.headers_mut().typed_insert(Connection::upgrade());
        res.headers_mut().typed_insert(Upgrade::websocket());
        res.headers_mut()
            .typed_insert(SecWebsocketAccept::from(self.key));

        if let Some(protocol) = protocol {
            res.headers_mut().insert(SEC_WEBSOCKET_PROTOCOL, protocol);
        }

        res
    }
}
