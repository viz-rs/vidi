//! Cookie attributes and utilities shared by cookie-aware middleware.

use std::time::Duration;

use crate::types::{Cookie, Cookies, SameSite};

/// Cookie attributes used when issuing a cookie.
#[derive(Debug)]
pub struct CookieOptions {
    /// The cookie name; `vidi.sid` by default.
    pub name: &'static str,
    /// The cookie path; `/` by default.
    pub path: &'static str,
    /// Whether the cookie is `Secure`; `true` by default.
    pub secure: bool,
    /// Whether the cookie is `HttpOnly`; `true` by default.
    pub http_only: bool,
    /// The cookie maximum age; 24 hours by default.
    pub max_age: Option<Duration>,
    /// The cookie domain.
    pub domain: Option<&'static str>,
    /// The cookie `SameSite`; `Lax` by default.
    pub same_site: Option<SameSite>,
}

impl CookieOptions {
    /// The default maximum cookie age: 24 hours.
    pub const MAX_AGE: u64 = 3600 * 24;

    /// Creates [`CookieOptions`] with the given name.
    #[must_use]
    pub fn new(name: &'static str) -> Self {
        Self::default().name(name)
    }

    /// Sets the cookie name.
    #[must_use]
    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Sets the cookie maximum age.
    #[must_use]
    pub fn max_age(mut self, max_age: Duration) -> Self {
        self.max_age.replace(max_age);
        self
    }

    /// Sets the cookie domain.
    #[must_use]
    pub fn domain(mut self, domain: &'static str) -> Self {
        self.domain.replace(domain);
        self
    }

    /// Sets the cookie path.
    #[must_use]
    pub const fn path(mut self, path: &'static str) -> Self {
        self.path = path;
        self
    }

    /// Sets whether the cookie is `Secure`.
    #[must_use]
    pub const fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Sets whether the cookie is `HttpOnly`.
    #[must_use]
    pub const fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Sets the cookie `SameSite`.
    #[must_use]
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site.replace(same_site);
        self
    }

    /// Converts self into a [`Cookie`].
    ///
    /// # Panics
    ///
    /// Panics if `std::time::Duration` cannot be converted to `cookie::time::Duration`.
    pub fn into_cookie(&self, value: impl Into<String>) -> Cookie<'_> {
        let mut cookie = Cookie::new(self.name, value.into());

        cookie.set_path(self.path);
        cookie.set_secure(self.secure);
        cookie.set_http_only(self.http_only);
        cookie.set_same_site(self.same_site);

        if let Some(domain) = self.domain {
            cookie.set_domain(domain);
        }
        if let Some(max_age) = self.max_age {
            cookie.set_max_age(
                ::cookie::time::Duration::try_from(max_age).expect(
                    "`std::time::Duration` cannot be converted to `cookie::time::Duration`",
                ),
            );
        }

        cookie
    }
}

impl Default for CookieOptions {
    fn default() -> Self {
        Self {
            domain: None,
            secure: true,
            http_only: true,
            path: "/",
            name: "vidi.sid",
            same_site: Some(SameSite::Lax),
            max_age: Some(Duration::from_secs(Self::MAX_AGE)),
        }
    }
}

macro_rules! impl_cookieable {
    ($doc:literal, $get:ident, $remove:ident, $add:ident) => {
        #[doc = $doc]
        pub trait Cookieable {
            /// Returns the cookie options.
            fn options(&self) -> &CookieOptions;

            /// Returns a cookie by name.
            fn get_cookie<'a>(&'a self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
                cookies.$get(self.options().name)
            }

            /// Removes a cookie.
            fn remove_cookie<'a>(&'a self, cookies: &'a Cookies) {
                cookies.$remove(self.options().name);
            }

            /// Sets a cookie.
            fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: impl Into<String>) {
                cookies.$add(self.options().into_cookie(value));
            }
        }
    };
}

#[cfg(not(feature = "cookie-private"))]
impl_cookieable!("An interface for managing the cookies.", get, remove, add);

#[cfg(feature = "cookie-private")]
impl_cookieable!(
    "An interface for managing private cookies.",
    private_get,
    private_remove,
    private_add
);
