//! CORS Middleware.

use std::{collections::HashSet, fmt, sync::Arc};

use crate::{
    header::{
        HeaderMap, HeaderName, HeaderValue, ACCESS_CONTROL_ALLOW_CREDENTIALS,
        ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_ORIGIN, ACCESS_CONTROL_REQUEST_HEADERS,
        ACCESS_CONTROL_REQUEST_METHOD, ORIGIN, VARY,
    },
    headers::{
        AccessControlAllowHeaders, AccessControlAllowMethods, AccessControlExposeHeaders,
        HeaderMapExt,
    },
    Handler, IntoResponse, Method, Request, RequestExt, Response, Result, StatusCode, Transform,
};

/// A configuration for [`CorsMiddleware`].
pub struct Config {
    max_age: usize,
    credentials: bool,
    allow_methods: HashSet<Method>,
    allow_headers: HashSet<HeaderName>,
    allow_origins: HashSet<HeaderValue>,
    expose_headers: HashSet<HeaderName>,
    origin_verify: Option<Arc<dyn Fn(&HeaderValue) -> bool + Send + Sync>>,
}

impl Config {
    /// Create a new [`Config`] with default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seconds a preflight request can be cached. [MDN]
    ///
    /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Max-Age
    #[must_use]
    pub fn max_age(mut self, max_age: usize) -> Self {
        self.max_age = max_age;
        self
    }

    /// Whether to allow credentials. [MDN]
    ///
    /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Credentials
    #[must_use]
    pub fn credentials(mut self, credentials: bool) -> Self {
        self.credentials = credentials;
        self
    }

    /// Allowed HTTP methods. [MDN]
    ///
    /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Methods
    #[must_use]
    pub fn allow_methods<H>(mut self, allow_methods: H) -> Self
    where
        H: IntoIterator,
        H::Item: TryInto<Method>,
    {
        self.allow_methods = allow_methods
            .into_iter()
            .map(TryInto::try_into)
            .filter_map(Result::ok)
            .collect();
        self
    }

    /// Allowed HTTP headers. [MDN]
    ///
    /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Headers
    #[must_use]
    pub fn allow_headers<H>(mut self, allow_headers: H) -> Self
    where
        H: IntoIterator,
        H::Item: TryInto<HeaderName>,
    {
        self.allow_headers = allow_headers
            .into_iter()
            .map(TryInto::try_into)
            .filter_map(Result::ok)
            .collect();
        self
    }

    /// Allowed origins. [MDN]
    ///
    /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Allow-Origin
    #[must_use]
    pub fn allow_origins<H>(mut self, allow_origins: H) -> Self
    where
        H: IntoIterator,
        H::Item: TryInto<HeaderValue>,
    {
        self.allow_origins = allow_origins
            .into_iter()
            .map(TryInto::try_into)
            .filter_map(Result::ok)
            .collect();
        self
    }

    /// Exposed HTTP headers. [MDN]
    ///
    /// [MDN]: https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Access-Control-Expose-Headers
    #[must_use]
    pub fn expose_headers<H>(mut self, expose_headers: H) -> Self
    where
        H: IntoIterator,
        H::Item: TryInto<HeaderName>,
    {
        self.expose_headers = expose_headers
            .into_iter()
            .map(TryInto::try_into)
            .filter_map(Result::ok)
            .collect();
        self
    }

    /// A function to verify the origin. If the function returns false, the request will be rejected.
    #[must_use]
    pub fn origin_verify(
        mut self,
        origin_verify: Option<Arc<dyn Fn(&HeaderValue) -> bool + Send + Sync>>,
    ) -> Self {
        self.origin_verify = origin_verify;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_age: 86400,
            credentials: false,
            allow_methods: HashSet::from([
                Method::GET,
                Method::POST,
                Method::HEAD,
                Method::PUT,
                Method::DELETE,
                Method::PATCH,
            ]),
            allow_origins: HashSet::from([HeaderValue::from_static("*")]),
            allow_headers: HashSet::new(),
            expose_headers: HashSet::new(),
            origin_verify: None,
        }
    }
}

impl Clone for Config {
    fn clone(&self) -> Self {
        Self {
            max_age: self.max_age,
            credentials: self.credentials,
            allow_methods: self.allow_methods.clone(),
            allow_headers: self.allow_headers.clone(),
            allow_origins: self.allow_origins.clone(),
            expose_headers: self.expose_headers.clone(),
            origin_verify: self.origin_verify.clone(),
        }
    }
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CorsConfig")
            .field("max_age", &self.max_age)
            .field("credentials", &self.credentials)
            .field("allow_methods", &self.allow_methods)
            .field("allow_headers", &self.allow_headers)
            .field("allow_origins", &self.allow_origins)
            .field("expose_headers", &self.expose_headers)
            .finish_non_exhaustive()
    }
}

impl<H> Transform<H> for Config {
    type Output = CorsMiddleware<H>;

    fn transform(&self, h: H) -> Self::Output {
        CorsMiddleware {
            h,
            acam: self.allow_methods.clone().into_iter().collect(),
            acah: self.allow_headers.clone().into_iter().collect(),
            aceh: self.expose_headers.clone().into_iter().collect(),
            config: self.clone(),
        }
    }
}

/// CORS middleware.
#[derive(Debug, Clone)]
pub struct CorsMiddleware<H> {
    h: H,
    config: Config,
    acam: AccessControlAllowMethods,
    acah: AccessControlAllowHeaders,
    aceh: AccessControlExposeHeaders,
}

#[crate::async_trait]
impl<H, O> Handler<Request> for CorsMiddleware<H>
where
    H: Handler<Request, Output = Result<O>> + Send + Clone + 'static,
    O: IntoResponse + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        let Some(origin) = req.header(ORIGIN).filter(is_not_empty) else {
            return self.h.call(req).await.map(IntoResponse::into_response);
        };

        if !self.config.allow_origins.contains(&origin)
            || !self
                .config
                .origin_verify
                .as_ref()
                .map_or(true, |f| (f)(&origin))
        {
            return Err(StatusCode::FORBIDDEN.into_error());
        }

        let mut headers = HeaderMap::new();
        let mut resp = if req.method() == Method::OPTIONS {
            // Preflight request
            if req
                .header(ACCESS_CONTROL_REQUEST_METHOD)
                .map_or(false, |method| {
                    self.config.allow_methods.is_empty()
                        || self.config.allow_methods.contains(&method)
                })
            {
                headers.typed_insert(self.acam.clone());
            } else {
                return Err((StatusCode::FORBIDDEN, "Invalid Preflight Request").into_error());
            }

            let (allow_headers, request_headers) = req
                .header(ACCESS_CONTROL_REQUEST_HEADERS)
                .map_or((true, None), |hs: HeaderValue| {
                    (
                        hs.to_str()
                            .map(|hs| {
                                hs.split(',')
                                    .map(str::as_bytes)
                                    .map(HeaderName::from_bytes)
                                    .filter_map(Result::ok)
                                    .any(|header| self.config.allow_headers.contains(&header))
                            })
                            .unwrap_or(false),
                        Some(hs),
                    )
                });

            if !allow_headers {
                return Err((StatusCode::FORBIDDEN, "Invalid Preflight Request").into_error());
            }

            if self.config.allow_headers.is_empty() {
                headers.insert(
                    ACCESS_CONTROL_ALLOW_HEADERS,
                    request_headers.unwrap_or(HeaderValue::from_static("*")),
                );
            } else {
                headers.typed_insert(self.acah.clone());
            }

            // 204 - no content
            StatusCode::NO_CONTENT.into_response()
        } else {
            // Simple Request
            if !self.config.expose_headers.is_empty() {
                headers.typed_insert(self.aceh.clone());
            }

            self.h.call(req).await.map(IntoResponse::into_response)?
        };

        // https://github.com/rs/cors/issues/10
        headers.insert(VARY, ORIGIN.into());
        headers.insert(ACCESS_CONTROL_ALLOW_ORIGIN, origin);

        if self.config.credentials {
            headers.insert(
                ACCESS_CONTROL_ALLOW_CREDENTIALS,
                HeaderValue::from_static("true"),
            );
        }

        resp.headers_mut().extend(headers);

        Ok(resp)
    }
}

fn is_not_empty(h: &HeaderValue) -> bool {
    !h.is_empty()
}
