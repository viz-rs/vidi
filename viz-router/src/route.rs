//! Route

use core::fmt;

use viz_core::{
    BoxHandler, Handler, HandlerExt, IntoResponse, Method, Next, Request, Response, Result,
    Transform,
};

macro_rules! export_internal_verb {
    ($name:ident $verb:tt) => {
        #[doc = concat!(" Appends a handler buy the HTTP `", stringify!($verb), "` verb into the route.")]
        #[must_use]
        pub fn $name<H, O>(self, handler: H) -> Self
        where
            H: Handler<Request, Output = Result<O>> + Clone,
            O: IntoResponse + Send + 'static,
        {
            self.on(Method::$verb, handler)
        }
    };
}

macro_rules! export_verb {
    ($name:ident $verb:ty) => {
        #[doc = concat!(" Creates a route with a handler and HTTP `", stringify!($verb), "` verb pair.")]
        #[must_use]
        pub fn $name<H, O>(handler: H) -> Route
        where
            H: Handler<Request, Output = Result<O>> + Clone,
            O: IntoResponse + Send + 'static,
        {
            Route::new().$name(handler)
        }
    };
}

/// A collection of verb-handler pair.
#[derive(Clone, Default)]
pub struct Route {
    pub(crate) methods: Vec<(Method, BoxHandler)>,
}

impl Route {
    /// Creates a new route.
    #[must_use]
    pub fn new() -> Self {
        Self {
            methods: Vec::new(),
        }
    }

    /// Appends a HTTP verb and handler pair into the route.
    #[must_use]
    pub fn push(mut self, method: Method, handler: BoxHandler) -> Self {
        match self
            .methods
            .iter_mut()
            .find(|(m, _)| m == method)
            .map(|(_, e)| e)
        {
            Some(h) => *h = handler,
            None => self.methods.push((method, handler)),
        }

        self
    }

    /// Appends a handler by the specified HTTP verb into the route.
    #[must_use]
    pub fn on<H, O>(self, method: Method, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse + Send + 'static,
    {
        self.push(method, handler.map_into_response().boxed())
    }

    /// Appends a handler by any HTTP verbs into the route.
    #[must_use]
    pub fn any<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse + Send + 'static,
    {
        [
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::HEAD,
            Method::OPTIONS,
            Method::CONNECT,
            Method::PATCH,
            Method::TRACE,
        ]
        .into_iter()
        .fold(self, |route, method| route.on(method, handler.clone()))
    }

    repeat!(
        export_internal_verb
        get GET
        post POST
        put PUT
        delete DELETE
        head HEAD
        options OPTIONS
        connect CONNECT
        patch PATCH
        trace TRACE
    );

    /// Takes a closure and creates an iterator which calls that closure on each handler.
    #[must_use]
    pub fn map_handler<F>(self, f: F) -> Self
    where
        F: Fn(BoxHandler) -> BoxHandler,
    {
        self.into_iter()
            .map(|(method, handler)| (method, f(handler)))
            .collect()
    }

    /// Transforms the types to a middleware and adds it.
    #[must_use]
    pub fn with<T>(self, t: T) -> Self
    where
        T: Transform<BoxHandler>,
        T::Output: Handler<Request, Output = Result<Response>>,
    {
        self.map_handler(|handler| t.transform(handler).boxed())
    }

    /// Adds a middleware for the routes.
    #[must_use]
    pub fn with_handler<H>(self, f: H) -> Self
    where
        H: Handler<Next<Request, BoxHandler>, Output = Result<Response>> + Clone,
    {
        self.map_handler(|handler| handler.around(f.clone()).boxed())
    }
}

impl IntoIterator for Route {
    type Item = (Method, BoxHandler);

    type IntoIter = std::vec::IntoIter<(Method, BoxHandler)>;

    fn into_iter(self) -> Self::IntoIter {
        self.methods.into_iter()
    }
}

impl FromIterator<(Method, BoxHandler)> for Route {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (Method, BoxHandler)>,
    {
        Self {
            methods: iter.into_iter().collect(),
        }
    }
}

/// Creates a route with a handler and HTTP verb pair.
pub fn on<H, O>(method: Method, handler: H) -> Route
where
    H: Handler<Request, Output = Result<O>> + Clone,
    O: IntoResponse + Send + 'static,
{
    Route::new().on(method, handler)
}

repeat!(
    export_verb
    get GET
    post POST
    put PUT
    delete DELETE
    head HEAD
    options OPTIONS
    connect CONNECT
    patch PATCH
    trace TRACE
);

/// Creates a route with a handler and any HTTP verbs.
pub fn any<H, O>(handler: H) -> Route
where
    H: Handler<Request, Output = Result<O>> + Clone,
    O: IntoResponse + Send + 'static,
{
    Route::new().any(handler)
}

impl fmt::Debug for Route {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Route")
            .field(
                "methods",
                &self
                    .methods
                    .iter()
                    .map(|(m, _)| m)
                    .collect::<Vec<&Method>>(),
            )
            .finish()
    }
}

#[cfg(test)]
#[allow(dead_code)]
#[allow(clippy::unused_async)]
mod tests {
    use super::Route;
    use http_body_util::BodyExt;
    use serde::Deserialize;
    use std::sync::Arc;
    use viz_core::{
        async_trait,
        handler::Transform,
        types::{Query, State},
        Handler, HandlerExt, IntoHandler, IntoResponse, Method, Next, Request, RequestExt,
        Response, Result,
    };

    #[tokio::test]
    async fn route() -> anyhow::Result<()> {
        async fn handler(_: Request) -> Result<impl IntoResponse> {
            Ok(())
        }

        struct Logger;

        impl Logger {
            fn new() -> Self {
                Self
            }
        }

        impl<H: Clone> Transform<H> for Logger {
            type Output = LoggerHandler<H>;

            fn transform(&self, h: H) -> Self::Output {
                LoggerHandler(h)
            }
        }

        #[derive(Clone)]
        struct LoggerHandler<H>(H);

        #[async_trait]
        impl<H> Handler<Request> for LoggerHandler<H>
        where
            H: Handler<Request> + Clone,
        {
            type Output = H::Output;

            async fn call(&self, req: Request) -> Self::Output {
                self.0.call(req).await
            }
        }

        async fn before(req: Request) -> Result<Request> {
            Ok(req)
        }

        async fn after(res: Result<Response>) -> Result<Response> {
            res
        }

        async fn around<H, O>((req, handler): Next<Request, H>) -> Result<Response>
        where
            H: Handler<Request, Output = Result<O>> + Clone,
            O: IntoResponse + Send + 'static,
        {
            handler.call(req).await.map(IntoResponse::into_response)
        }

        async fn around_1<H, O>((req, handler): Next<Request, H>) -> Result<Response>
        where
            H: Handler<Request, Output = Result<O>> + Clone,
            O: IntoResponse + Send + 'static,
        {
            handler.call(req).await.map(IntoResponse::into_response)
        }

        async fn around_2<H>((req, handler): Next<Request, H>) -> Result<Response>
        where
            H: Handler<Request, Output = Result<Response>> + Clone,
        {
            handler.call(req).await
        }

        #[derive(Clone)]
        struct Around2 {
            name: String,
        }

        #[async_trait]
        impl<H, I, O> Handler<Next<I, H>> for Around2
        where
            I: Send + 'static,
            H: Handler<I, Output = Result<O>> + Clone,
        {
            type Output = H::Output;

            async fn call(&self, (i, h): Next<I, H>) -> Self::Output {
                h.call(i).await
            }
        }

        #[derive(Clone)]
        struct Around3 {
            name: String,
        }

        #[async_trait]
        impl<H, O> Handler<Next<Request, H>> for Around3
        where
            H: Handler<Request, Output = Result<O>> + Clone,
            O: IntoResponse,
        {
            type Output = Result<Response>;

            async fn call(&self, (i, h): Next<Request, H>) -> Self::Output {
                h.call(i).await.map(IntoResponse::into_response)
            }
        }

        #[derive(Clone)]
        struct Around4 {
            name: String,
        }

        #[async_trait]
        impl<H> Handler<Next<Request, H>> for Around4
        where
            H: Handler<Request, Output = Result<Response>> + Clone,
        {
            type Output = Result<Response>;

            async fn call(&self, (i, h): Next<Request, H>) -> Self::Output {
                h.call(i).await
            }
        }

        #[derive(Deserialize)]
        struct Counter {
            c: u8,
        }

        async fn ext(q: Query<Counter>, s: State<Arc<String>>) -> Result<impl IntoResponse> {
            let mut a = s.to_string().as_bytes().to_vec();
            a.push(q.c);
            Ok(a)
        }

        let route = Route::new()
            .any(ext.into_handler())
            .on(Method::GET, handler.map_into_response().before(before))
            .on(Method::POST, handler.map_into_response().after(after))
            .put(handler.around(Around2 {
                name: "handler around".to_string(),
            }))
            .with(Logger::new())
            .map_handler(|handler| {
                handler
                    .before(|mut req: Request| async {
                        req.set_state(Arc::new("before".to_string()));
                        Ok(req)
                    })
                    .before(before)
                    .around(around_2)
                    .after(after)
                    .around(Around4 {
                        name: "4".to_string(),
                    })
                    .around(Around2 {
                        name: "2".to_string(),
                    })
                    .around(around)
                    .around(around_1)
                    .around(Around3 {
                        name: "3".to_string(),
                    })
                    .with(Logger::new())
                    // .boxed()
                    // .boxed()
                    .boxed()
            })
            .with_handler(around)
            .with_handler(around_1)
            .with_handler(around_2)
            .with_handler(Around2 {
                name: "2 with handler".to_string(),
            })
            .with_handler(Around3 {
                name: "3 with handler".to_string(),
            })
            .with_handler(Around4 {
                name: "4 with handler".to_string(),
            })
            // .with(viz_core::middleware::cookie::Config::new())
            .into_iter()
            // .filter(|(method, _)| method != Method::GET)
            .collect::<Route>();

        let (_, h) = route
            .methods
            .iter()
            .find(|(m, _)| m == Method::GET)
            .unwrap();

        let resp = match h.call(Request::default()).await {
            Ok(r) => r,
            Err(e) => e.into_response(),
        };
        assert_eq!(resp.into_body().collect().await?.to_bytes(), "");

        let (_, h) = route
            .methods
            .iter()
            .find(|(m, _)| m == Method::DELETE)
            .unwrap();

        let mut req = Request::default();
        *req.uri_mut() = "/?c=1".parse().unwrap();

        let resp = match h.call(req).await {
            Ok(r) => r,
            Err(e) => e.into_response(),
        };
        assert_eq!(
            resp.into_body().collect().await?.to_bytes().to_vec(),
            vec![98, 101, 102, 111, 114, 101, 1]
        );

        Ok(())
    }
}
