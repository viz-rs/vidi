//! Represents a cookie-jar extractor.

use std::{
    fmt,
    sync::{Arc, Mutex},
};

use crate::{
    Error, FromRequest, IntoResponse, Request, RequestExt, Response, StatusCode, ThisError,
};

pub use ::cookie::{Cookie, CookieJar, SameSite};

/// A cryptographic master key for use with `Signed` and/or `Private` jars.
#[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
pub type CookieKey = ::cookie::Key;

/// Extracts the cookies from the request.
pub struct Cookies {
    inner: Arc<Mutex<CookieJar>>,
    #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
    key: Option<Arc<CookieKey>>,
}

impl Clone for Cookies {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
            key: self.key.clone(),
        }
    }
}

impl Cookies {
    /// Creates a new Cookies with the [`CookieJar`].
    #[must_use]
    pub fn new(cookie_jar: CookieJar) -> Self {
        Self {
            inner: Arc::new(Mutex::new(cookie_jar)),
            #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
            key: None,
        }
    }

    /// Retures the inner mutex [`CookieJar`].
    #[must_use]
    pub fn jar(&self) -> &Mutex<CookieJar> {
        &self.inner
    }

    /// Removes `cookie` from this cookies.
    pub fn remove(&self, name: impl AsRef<str>) {
        if let Ok(mut c) = self.jar().lock() {
            c.remove(Cookie::from(name.as_ref().to_string()));
        }
    }

    /// Returns a `Cookie` inside this cookies with the name.
    pub fn get(&self, name: impl AsRef<str>) -> Option<Cookie<'_>> {
        self.jar()
            .lock()
            .ok()
            .and_then(|c| c.get(name.as_ref()).cloned())
    }

    /// Adds `cookie` to this cookies.
    pub fn add(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.add(cookie.into_owned());
        }
    }

    /// Adds an "original" `cookie` to this cookies.
    pub fn add_original(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.add_original(cookie.into_owned());
        }
    }

    /// Removes all delta cookies.
    pub fn reset_delta(&self) {
        if let Ok(mut c) = self.jar().lock() {
            c.reset_delta();
        }
    }
}

#[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
impl Cookies {
    /// A cryptographic master key for use with `Signed` and/or `Private` jars.
    #[must_use]
    pub fn with_key(mut self, key: Arc<CookieKey>) -> Self {
        self.key.replace(key);
        self
    }

    /// Retures the cryptographic master [`Key`][CookieKey].
    ///
    /// # Panics
    ///
    /// Will panic if missing a key
    #[must_use]
    pub fn key(&self) -> &CookieKey {
        self.key.as_ref().expect("the `CookieKey` is required")
    }
}

#[cfg(feature = "cookie-private")]
impl Cookies {
    /// Returns a reference to the `Cookie` inside this jar with the specified name.
    pub fn private_get(&self, name: impl AsRef<str>) -> Option<Cookie<'_>> {
        self.jar()
            .lock()
            .ok()
            .and_then(|c| c.private(self.key()).get(name.as_ref()))
    }

    /// Adds `cookie` to the parent jar.
    pub fn private_add(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.private_mut(self.key()).add(cookie.into_owned());
        }
    }

    /// Removes `cookie` from the parent jar.
    pub fn private_remove(&self, name: impl AsRef<str>) {
        if let Ok(mut c) = self.jar().lock() {
            c.private_mut(self.key())
                .remove(Cookie::from(name.as_ref().to_string()));
        }
    }

    /// Adds an "original" `cookie` to parent jar.
    pub fn private_add_original(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.private_mut(self.key()).add_original(cookie.into_owned());
        }
    }

    /// Authenticates and decrypts `cookie` and returning the plain `cookie`.
    #[must_use]
    pub fn private_decrypt(&self, cookie: Cookie<'_>) -> Option<Cookie<'_>> {
        self.jar()
            .lock()
            .ok()?
            .private(self.key())
            .decrypt(cookie.into_owned())
    }
}

#[cfg(feature = "cookie-signed")]
impl Cookies {
    /// Returns a reference to the `Cookie` inside this jar with the specified name.
    pub fn signed_get(&self, name: impl AsRef<str>) -> Option<Cookie<'_>> {
        self.jar()
            .lock()
            .ok()
            .and_then(|c| c.signed(self.key()).get(name.as_ref()))
    }

    /// Adds `cookie` to the parent jar.
    pub fn signed_add(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.signed_mut(self.key()).add(cookie.into_owned());
        }
    }

    /// Removes `cookie` from the parent jar.
    pub fn signed_remove(&self, name: impl AsRef<str>) {
        if let Ok(mut c) = self.jar().lock() {
            c.signed_mut(self.key())
                .remove(Cookie::from(name.as_ref().to_string()));
        }
    }

    /// Adds an "original" `cookie` to parent jar.
    pub fn signed_add_original(&self, cookie: Cookie<'_>) {
        if let Ok(mut c) = self.jar().lock() {
            c.signed_mut(self.key()).add_original(cookie.into_owned());
        }
    }

    /// Verifies the authenticity and integrity of `cookie` and returning the plain `cookie`.
    #[must_use]
    pub fn signed_verify(&self, cookie: Cookie<'_>) -> Option<Cookie<'_>> {
        self.jar()
            .lock()
            .ok()?
            .signed(self.key())
            .verify(cookie.into_owned())
    }
}

#[crate::async_trait]
impl FromRequest for Cookies {
    type Error = CookiesError;

    async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
        req.cookies()
    }
}

impl fmt::Debug for Cookies {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut d = f.debug_struct("Cookies");

        d.field("jar", self.inner.as_ref());

        #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
        d.field("key", &self.key.is_some());

        d.finish()
    }
}

/// Rejects a error thats reading or parsing the cookies.
#[derive(ThisError, Debug)]
pub enum CookiesError {
    /// Failed to read cookies
    #[error("failed to read cookies")]
    Read,
    /// Failed to parse cookies
    #[error("failed to parse cookies")]
    Parse,
}

impl From<CookiesError> for Error {
    fn from(e: CookiesError) -> Self {
        Error::Responder(e.into_response())
    }
}

impl IntoResponse for CookiesError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
