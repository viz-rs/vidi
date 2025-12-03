//! Cookie helper.

use std::time::Duration;

use crate::types::{Cookie, Cookies, SameSite};

/// Cookie's Options
#[derive(Debug)]
pub struct CookieOptions {
    /// Cookie's `name`, `viz.sid` by defaults
    pub name: &'static str,
    /// Cookie's `path`, `/` by defaults
    pub path: &'static str,
    /// Cookie's `secure`, `true` by defaults
    pub secure: bool,
    /// Cookie's `http_only`, `true` by defaults
    pub http_only: bool,
    /// Cookie's maximum age, `24H` by defaults
    pub max_age: Option<Duration>,
    /// Cookie's `domain`
    pub domain: Option<&'static str>,
    /// Cookie's `same_site`, `Lax` by defaults
    pub same_site: Option<SameSite>,
}

impl CookieOptions {
    /// By default 24h for cookie.
    pub const MAX_AGE: u64 = 3600 * 24;

    /// Creates new `CookieOptions`
    #[must_use]
    pub fn new(name: &'static str) -> Self {
        Self::default().name(name)
    }

    /// Creates new `CookieOptions` with `name`
    #[must_use]
    pub const fn name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Creates new `CookieOptions` with `max_age`
    #[must_use]
    pub fn max_age(mut self, max_age: Duration) -> Self {
        self.max_age.replace(max_age);
        self
    }

    /// Creates new `CookieOptions` with `domain`
    #[must_use]
    pub fn domain(mut self, domain: &'static str) -> Self {
        self.domain.replace(domain);
        self
    }

    /// Creates new `CookieOptions` with `path`
    #[must_use]
    pub const fn path(mut self, path: &'static str) -> Self {
        self.path = path;
        self
    }

    /// Creates new `CookieOptions` with `secure`
    #[must_use]
    pub const fn secure(mut self, secure: bool) -> Self {
        self.secure = secure;
        self
    }

    /// Creates new `CookieOptions` with `http_only`
    #[must_use]
    pub const fn http_only(mut self, http_only: bool) -> Self {
        self.http_only = http_only;
        self
    }

    /// Creates new `CookieOptions` with `same_site`
    #[must_use]
    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site.replace(same_site);
        self
    }

    /// Converts self into a [Cookie].
    ///
    /// # Panics
    ///
    /// Will panic if `std::time::Duration` cannot be converted to `cookie::ime::Duration`
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
            cookie
                .set_max_age(::cookie::time::Duration::try_from(max_age).expect(
                    "`std::time::Duration` cannot be converted to `cookie::ime::Duration`",
                ));
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

/// An interface for managing the cookies.
#[cfg(not(feature = "cookie-private"))]
pub trait Cookieable {
    /// Gets the options of the cookie.
    fn options(&self) -> &CookieOptions;

    /// Gets a cookie from the cookies.
    fn get_cookie<'a>(&'a self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        cookies.get(self.options().name)
    }

    /// Deletes a cookie from the cookies.
    fn remove_cookie<'a>(&'a self, cookies: &'a Cookies) {
        cookies.remove(self.options().name);
    }

    /// Sets a cookie from the cookies.
    fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: impl Into<String>) {
        cookies.add(self.options().into_cookie(value));
    }
}

/// An interface for managing the `private` cookies.
#[cfg(feature = "cookie-private")]
pub trait Cookieable {
    /// Gets the options of the cookie.
    fn options(&self) -> &CookieOptions;

    /// Gets a cookie from the cookies.
    fn get_cookie<'a>(&'a self, cookies: &'a Cookies) -> Option<Cookie<'a>> {
        cookies.private_get(self.options().name)
    }

    /// Deletes a cookie from the cookies.
    fn remove_cookie<'a>(&'a self, cookies: &'a Cookies) {
        cookies.private_remove(self.options().name);
    }

    /// Sets a cookie from the cookies.
    fn set_cookie<'a>(&'a self, cookies: &'a Cookies, value: impl Into<String>) {
        cookies.private_add(self.options().into_cookie(value));
    }
}
