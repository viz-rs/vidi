//! Limits Middleware.

#[cfg(feature = "multipart")]
use std::sync::Arc;

use crate::{async_trait, types, Handler, IntoResponse, Request, Response, Result, Transform};

/// A configuration for [`LimitsMiddleware`].
#[derive(Debug, Clone)]
pub struct Config {
    limits: types::Limits,
    #[cfg(feature = "multipart")]
    multipart: Arc<types::MultipartLimits>,
}

impl Config {
    /// Creates a new Config.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a limits for the Text/Bytes/Form.
    #[must_use]
    pub fn limits(mut self, limits: types::Limits) -> Self {
        self.limits = limits.sort();
        self
    }

    /// Sets a limits for the Multipart Form.
    #[cfg(feature = "multipart")]
    #[must_use]
    pub fn multipart(mut self, limits: types::MultipartLimits) -> Self {
        *Arc::make_mut(&mut self.multipart) = limits;
        self
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            limits: types::Limits::default(),
            #[cfg(feature = "multipart")]
            multipart: Arc::new(types::MultipartLimits::default()),
        }
    }
}

impl<H> Transform<H> for Config
where
    H: Clone,
{
    type Output = LimitsMiddleware<H>;

    fn transform(&self, h: H) -> Self::Output {
        LimitsMiddleware {
            h,
            config: self.clone(),
        }
    }
}

/// Limits middleware.
#[derive(Debug, Clone)]
pub struct LimitsMiddleware<H> {
    h: H,
    config: Config,
}

#[async_trait]
impl<H, O> Handler<Request> for LimitsMiddleware<H>
where
    O: IntoResponse,
    H: Handler<Request, Output = Result<O>> + Clone,
{
    type Output = Result<Response>;

    async fn call(&self, mut req: Request) -> Self::Output {
        req.extensions_mut().insert(self.config.limits.clone());
        #[cfg(feature = "multipart")]
        req.extensions_mut().insert(self.config.multipart.clone());

        self.h.call(req).await.map(IntoResponse::into_response)
    }
}
