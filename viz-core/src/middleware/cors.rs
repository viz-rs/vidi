//! CORS Middleware.

use std::{collections::HashSet, fmt, sync::Arc};

use crate::{
    async_trait,
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

/// A configuration for [CorsMiddleware].
pub struct Config {
    max_age: usize,
    credentials: bool,
    allow_methods: HashSet<Method>,
    allow_headers: HashSet<HeaderName>,
    allow_origins: HashSet<HeaderValue>,
    expose_headers: HashSet<HeaderName>,
    #[allow(clippy::type_complexity)]
    origin_verify: Option<Arc<dyn Fn(&HeaderValue) -> bool + Send + Sync>>,
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
            .finish()
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

#[async_trait]
impl<H, O> Handler<Request> for CorsMiddleware<H>
where
    O: IntoResponse,
    H: Handler<Request, Output = Result<O>> + Clone,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        let origin = match req.header(ORIGIN).filter(is_not_empty) {
            Some(origin) => origin,
            None => return self.h.call(req).await.map(IntoResponse::into_response),
        };

        if !self.config.allow_origins.contains(&origin)
            || !self
                .config
                .origin_verify
                .as_ref()
                .map(|f| (f)(&origin))
                .unwrap_or(true)
        {
            return Err(StatusCode::FORBIDDEN.into_error());
        }

        let mut headers = HeaderMap::new();
        let mut res = if req.method() == Method::OPTIONS {
            // Preflight request
            if req
                .header(ACCESS_CONTROL_REQUEST_METHOD)
                .map(|method| {
                    self.config.allow_methods.is_empty()
                        || self.config.allow_methods.contains(&method)
                })
                .unwrap_or(false)
            {
                headers.typed_insert(self.acam.clone());
            } else {
                return Err((StatusCode::FORBIDDEN, "Invalid Preflight Request").into_error());
            }

            let (allow_headers, request_headers) = req
                .header(ACCESS_CONTROL_REQUEST_HEADERS)
                .map(|hs: HeaderValue| {
                    (
                        hs.to_str()
                            .map(|hs| {
                                hs.split(',')
                                    .filter_map(|h| HeaderName::from_bytes(h.as_bytes()).ok())
                                    .any(|header| self.config.allow_headers.contains(&header))
                            })
                            .unwrap_or(false),
                        Some(hs),
                    )
                })
                .unwrap_or((true, None));

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

            self.h
                .call(req)
                .await
                .map_or_else(IntoResponse::into_response, IntoResponse::into_response)
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

        res.headers_mut().extend(headers);

        Ok(res)
    }
}

fn is_not_empty(h: &HeaderValue) -> bool {
    !h.is_empty()
}
