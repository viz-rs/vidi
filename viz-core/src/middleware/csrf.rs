//! CSRF Middleware.

use std::{collections::HashSet, fmt, sync::Arc};

use crate::{
    async_trait,
    header::{HeaderName, HeaderValue, VARY},
    middleware::helper::{CookieOptions, Cookieable},
    Error, FromRequest, Handler, IntoResponse, Method, Request, RequestExt, Response, Result,
    StatusCode, Transform,
};

struct Inner<S, G, V> {
    store: Store,
    ignored_methods: HashSet<Method>,
    cookie_options: CookieOptions,
    header: HeaderName,
    secret: S,
    generate: G,
    verify: V,
}

/// The CSRF token source that is cookie or session.
#[derive(Debug)]
pub enum Store {
    /// Via Cookie.
    Cookie,
    #[cfg(feature = "session")]
    /// Via Session.
    Session,
}

/// Extracts CSRF token via cookie or session.
#[derive(Debug, Clone)]
pub struct CsrfToken(pub String);

#[async_trait]
impl FromRequest for CsrfToken {
    type Error = Error;

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.extensions()
            .get()
            .cloned()
            .ok_or_else(|| (StatusCode::FORBIDDEN, "Missing csrf token").into_error())
    }
}

/// A configuration for [CsrfMiddleware].
pub struct Config<S, G, V>(Arc<Inner<S, G, V>>);

impl<S, G, V> Config<S, G, V> {
    /// The name of CSRF header.
    pub const CSRF_TOKEN: &'static str = "x-csrf-token";

    /// Creates a new configuration.
    pub fn new(
        store: Store,
        ignored_methods: HashSet<Method>,
        cookie_options: CookieOptions,
        secret: S,
        generate: G,
        verify: V,
    ) -> Self {
        Self(Arc::new(Inner {
            store,
            ignored_methods,
            cookie_options,
            secret,
            generate,
            verify,
            header: HeaderName::from_static(Self::CSRF_TOKEN),
        }))
    }

    /// Gets the CSRF token from cookies or session.
    pub fn get(&self, req: &Request) -> Result<Option<Vec<u8>>> {
        let inner = self.as_ref();
        match inner.store {
            Store::Cookie => {
                match self
                    .get_cookie(&req.cookies()?)
                    .map(|c| c.value().to_string())
                {
                    None => Ok(None),
                    Some(raw_token) => {
                        match base64::decode_config(raw_token, base64::URL_SAFE_NO_PAD) {
                            Ok(masked_token) if is_64(&masked_token) => {
                                Ok(Some(unmask::<32>(masked_token)))
                            }
                            _ => Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid csrf token")
                                .into_error()),
                        }
                    }
                }
            }
            #[cfg(feature = "session")]
            Store::Session => req.session().get(inner.cookie_options.name),
        }
    }

    /// Sets the CSRF token to cookies or session.
    #[allow(unused)]
    pub fn set(&self, req: &Request, token: String, secret: Vec<u8>) -> Result<()> {
        let inner = self.as_ref();
        match inner.store {
            Store::Cookie => {
                self.set_cookie(&req.cookies()?, token);
                Ok(())
            }
            #[cfg(feature = "session")]
            Store::Session => req.session().set(inner.cookie_options.name, secret),
        }
    }
}

impl<S, G, V> Clone for Config<S, G, V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<S, G, V> Cookieable for Config<S, G, V> {
    fn options(&self) -> &CookieOptions {
        &self.0.cookie_options
    }
}

impl<S, G, V> AsRef<Inner<S, G, V>> for Config<S, G, V> {
    fn as_ref(&self) -> &Inner<S, G, V> {
        &self.0
    }
}

impl<S, G, V> fmt::Debug for Config<S, G, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CsrfConfig")
            .field("header", &self.as_ref().header)
            .field("cookie_options", &self.as_ref().cookie_options)
            .field("ignored_methods", &self.as_ref().ignored_methods)
            .finish()
    }
}

impl<H, S, G, V> Transform<H> for Config<S, G, V> {
    type Output = CsrfMiddleware<H, S, G, V>;

    fn transform(&self, h: H) -> Self::Output {
        CsrfMiddleware {
            h,
            config: self.clone(),
        }
    }
}

/// CSRF middleware.
#[derive(Debug)]
pub struct CsrfMiddleware<H, S, G, V> {
    h: H,
    config: Config<S, G, V>,
}

impl<H, S, G, V> Clone for CsrfMiddleware<H, S, G, V>
where
    H: Clone,
{
    fn clone(&self) -> Self {
        Self {
            h: self.h.clone(),
            config: self.config.clone(),
        }
    }
}

#[async_trait]
impl<H, O, S, G, V> Handler<Request> for CsrfMiddleware<H, S, G, V>
where
    O: IntoResponse,
    H: Handler<Request, Output = Result<O>> + Clone,
    S: Fn() -> Result<Vec<u8>> + Send + Sync + 'static,
    G: Fn(&[u8], Vec<u8>) -> Vec<u8> + Send + Sync + 'static,
    V: Fn(Vec<u8>, String) -> bool + Send + Sync + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, mut req: Request) -> Self::Output {
        let mut secret = self.config.get(&req)?;
        let config = self.config.as_ref();

        if !config.ignored_methods.contains(req.method()) {
            let mut forbidden = true;
            if let Some(secret) = secret.take() {
                if let Some(raw_token) = req.header(&config.header) {
                    forbidden = !(config.verify)(secret, raw_token);
                }
            }
            if forbidden {
                return Err((StatusCode::FORBIDDEN, "Invalid csrf token").into_error());
            }
        }

        let otp = (config.secret)()?;
        let secret = (config.secret)()?;
        let token = base64::encode_config((config.generate)(&secret, otp), base64::URL_SAFE_NO_PAD);
        req.extensions_mut().insert(CsrfToken(token.to_string()));
        self.config.set(&req, token, secret)?;

        self.h
            .call(req)
            .await
            .map(IntoResponse::into_response)
            .map(|mut res| {
                res.headers_mut()
                    .insert(VARY, HeaderValue::from_static("Cookie"));
                res
            })
    }
}

/// Gets random secret
pub fn secret() -> Result<Vec<u8>> {
    let mut buf = [0u8; 32];
    getrandom::getrandom(&mut buf)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_error())?;
    Ok(buf.to_vec())
}

/// Generates Token
pub fn generate(secret: &[u8], otp: Vec<u8>) -> Vec<u8> {
    mask(secret.to_vec(), otp)
}

/// Verifys Token with a secret
pub fn verify(secret: Vec<u8>, raw_token: String) -> bool {
    if let Ok(token) = base64::decode_config(raw_token, base64::URL_SAFE_NO_PAD) {
        return is_64(&token) && secret == unmask::<32>(token);
    }
    false
}

/// Retures masked token
fn mask(secret: Vec<u8>, mut otp: Vec<u8>) -> Vec<u8> {
    otp.extend::<Vec<u8>>(
        secret
            .iter()
            .enumerate()
            .map(|(i, t)| *t ^ otp[i])
            .collect(),
    );
    otp
}

/// Returens secret
fn unmask<const N: usize>(mut token: Vec<u8>) -> Vec<u8> {
    // encrypted_csrf_token
    let mut secret = token.split_off(N);
    // one_time_pad
    secret
        .iter_mut()
        .enumerate()
        .for_each(|(i, t)| *t ^= token[i]);
    secret
}

fn is_64(buf: &Vec<u8>) -> bool {
    buf.len() == 64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn builder() {
        Config::new(
            Store::Cookie,
            [Method::GET, Method::HEAD, Method::OPTIONS, Method::TRACE].into(),
            CookieOptions::new("_csrf").max_age(Duration::from_secs(3600 * 24)),
            secret,
            generate,
            verify,
        );
    }
}
