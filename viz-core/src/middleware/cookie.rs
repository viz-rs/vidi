//! Cookie Middleware.

use std::fmt;

use crate::{
    async_trait,
    header::{HeaderValue, COOKIE, SET_COOKIE},
    types, Handler, IntoResponse, Request, Response, Result, Transform,
};

/// A configure for [`CookieMiddleware`].
pub struct Config {
    #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
    key: std::sync::Arc<types::CookieKey>,
}

#[allow(clippy::new_without_default)]
impl Config {
    /// Creates a new config.
    #[cfg(not(any(feature = "cookie-signed", feature = "cookie-private")))]
    #[must_use]
    pub fn new() -> Self {
        Self {}
    }

    /// Creates a new config with the [`Key`][types::CookieKey].
    #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
    pub fn new(key: types::CookieKey) -> Self {
        Self {
            key: std::sync::Arc::new(key),
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
    key: std::sync::Arc<types::CookieKey>,
}

impl<H> fmt::Debug for CookieMiddleware<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("CookieMiddleware");

        #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
        d.field("key", &[..]);

        d.finish()
    }
}

#[async_trait]
impl<H, O> Handler<Request> for CookieMiddleware<H>
where
    O: IntoResponse,
    H: Handler<Request, Output = Result<O>> + Clone,
{
    type Output = Result<Response>;

    async fn call(&self, mut req: Request) -> Self::Output {
        let jar = req
            .headers()
            .get_all(COOKIE)
            .iter()
            .filter_map(|c| HeaderValue::to_str(c).ok())
            .fold(types::CookieJar::new(), add_cookie);

        let cookies = types::Cookies::new(jar);
        #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
        let cookies = cookies.with_key(self.key.clone());

        req.extensions_mut()
            .insert::<types::Cookies>(cookies.clone());

        self.h
            .call(req)
            .await
            .map(IntoResponse::into_response)
            .map(|mut res| {
                if let Ok(c) = cookies.jar().lock() {
                    c.delta()
                        .filter_map(|cookie| {
                            HeaderValue::from_str(&cookie.encoded().to_string()).ok()
                        })
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
fn add_cookie(mut jar: types::CookieJar, value: &str) -> types::CookieJar {
    types::Cookie::split_parse_encoded(value)
        .filter_map(Result::ok)
        .for_each(|cookie| jar.add_original(cookie.into_owned()));
    jar
}
