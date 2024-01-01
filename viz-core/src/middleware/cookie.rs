//! Cookie Middleware.

use std::fmt;

use crate::{
    header::{HeaderValue, COOKIE, SET_COOKIE},
    types::{Cookie, CookieJar, CookieKey, Cookies},
    Handler, IntoResponse, Request, Response, Result, Transform,
};

/// A configure for [`CookieMiddleware`].
pub struct Config {
    #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
    key: std::sync::Arc<CookieKey>,
}

impl Config {
    /// Creates a new config with the [`Key`][CookieKey].
    #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
    #[must_use]
    pub fn with_key(key: CookieKey) -> Self {
        Self {
            key: std::sync::Arc::new(key),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
            key: std::sync::Arc::new(CookieKey::generate()),
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("CookieConfig");

        #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
        d.field("key", &[..]);

        d.finish()
    }
}

impl<H> Transform<H> for Config
where
    H: Clone,
{
    type Output = CookieMiddleware<H>;

    fn transform(&self, h: H) -> Self::Output {
        CookieMiddleware {
            h,
            #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
            key: self.key.clone(),
        }
    }
}

/// Cookie middleware.
#[derive(Clone)]
pub struct CookieMiddleware<H> {
    h: H,
    #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
    key: std::sync::Arc<CookieKey>,
}

impl<H> fmt::Debug for CookieMiddleware<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("CookieMiddleware");

        #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
        d.field("key", &[..]);

        d.finish()
    }
}

#[crate::async_trait]
impl<H, O> Handler<Request> for CookieMiddleware<H>
where
    H: Handler<Request, Output = Result<O>>,
    O: IntoResponse + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, mut req: Request) -> Self::Output {
        let jar = req
            .headers()
            .get_all(COOKIE)
            .iter()
            .map(HeaderValue::to_str)
            .filter_map(Result::ok)
            .fold(CookieJar::new(), add_cookie);

        let cookies = Cookies::new(jar);
        #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
        let cookies = cookies.with_key(self.key.clone());

        req.extensions_mut().insert::<Cookies>(cookies.clone());

        self.h
            .call(req)
            .await
            .map(IntoResponse::into_response)
            .map(|mut res| {
                if let Ok(c) = cookies.jar().lock() {
                    c.delta()
                        .map(Cookie::encoded)
                        .map(|cookie| HeaderValue::from_str(&cookie.to_string()))
                        .filter_map(Result::ok)
                        .fold(res.headers_mut(), |headers, cookie| {
                            headers.append(SET_COOKIE, cookie);
                            headers
                        });
                }
                res
            })
    }
}

#[inline]
fn add_cookie(mut jar: CookieJar, value: &str) -> CookieJar {
    Cookie::split_parse_encoded(value)
        .filter_map(Result::ok)
        .map(Cookie::into_owned)
        .for_each(|cookie| jar.add_original(cookie));
    jar
}
