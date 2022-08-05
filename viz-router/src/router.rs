use viz_core::{
    BoxHandler, Handler, HandlerExt, IntoResponse, Next, Request, Response, Result, Transform,
};

use crate::{Resources, Route};

macro_rules! export_verb {
    ($name:ident $verb:ty) => {
        #[doc = concat!(" Adds a handler with a path and HTTP `", stringify!($verb), "` verb pair.")]
        pub fn $name<S, H, O>(self, path: S, handler: H) -> Self
        where
            S: AsRef<str>,
            H: Handler<Request, Output = Result<O>> + Clone,
            O: IntoResponse + Send + Sync + 'static,
        {
            self.route(path, Route::new().$name(handler))
        }
    };
}

/// A routes collection.
#[derive(Clone, Debug, Default)]
pub struct Router {
    pub(crate) routes: Option<Vec<(String, Route)>>,
}

impl Router {
    /// Creates an empty `Router`.
    pub fn new() -> Self {
        Self { routes: None }
    }

    fn push<S>(routes: &mut Vec<(String, Route)>, path: S, route: Route)
    where
        S: AsRef<str>,
    {
        let path = path.as_ref();
        match routes
            .iter_mut()
            .find_map(|(p, r)| if p == path { Some(r) } else { None })
        {
            Some(r) => {
                *r = route.into_iter().fold(
                    // original route
                    r.clone().into_iter().collect(),
                    |or: Route, (method, handler)| or.on(method, handler),
                );
            }
            None => routes.push((path.to_string(), route)),
        }
    }

    /// Inserts a path-route pair into the router.
    pub fn route<S>(mut self, path: S, route: Route) -> Self
    where
        S: AsRef<str>,
    {
        Self::push(
            self.routes.get_or_insert_with(Vec::new),
            path.as_ref().trim_start_matches('/'),
            route,
        );
        self
    }

    /// Nested resources with a path.
    pub fn resources<S>(self, path: S, resource: Resources) -> Self
    where
        S: AsRef<str>,
    {
        let mut path = path.as_ref().to_string();
        if !path.ends_with('/') {
            path.push('/');
        }

        resource.into_iter().fold(self, |router, (mut sp, route)| {
            let is_empty = sp.is_empty();
            sp = path.clone() + &sp;
            if is_empty {
                sp = sp.trim_end_matches('/').to_string();
            }
            router.route(sp, route)
        })
    }

    /// Nested sub-router with a path.
    pub fn nest<S>(self, path: S, router: Self) -> Self
    where
        S: AsRef<str>,
    {
        let mut path = path.as_ref().to_string();
        if !path.ends_with('/') {
            path.push('/');
        }

        match router.routes {
            Some(routes) => routes.into_iter().fold(self, |router, (mut sp, route)| {
                let is_empty = sp.is_empty();
                sp = path.clone() + &sp;
                if is_empty {
                    sp = sp.trim_end_matches('/').to_string();
                }
                router.route(sp, route)
            }),
            None => self,
        }
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

    /// Adds a handler with a path and any HTTP verbs."
    pub fn any<S, H, O>(self, path: S, handler: H) -> Self
    where
        S: AsRef<str>,
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse + Send + Sync + 'static,
    {
        self.route(path, Route::new().any(handler))
    }

    /// Takes a closure and creates an iterator which calls that closure on each handler.
    pub fn map_handler<F>(self, f: F) -> Self
    where
        F: Fn(BoxHandler) -> BoxHandler,
    {
        Self {
            routes: self.routes.map(|routes| {
                routes
                    .into_iter()
                    .map(|(path, route)| {
                        (
                            path,
                            route
                                .into_iter()
                                .map(|(method, handler)| (method, f(handler)))
                                .collect(),
                        )
                    })
                    .collect()
            }),
        }
    }

    /// Transforms the types to a middleware and adds it.
    pub fn with<T>(self, t: T) -> Self
    where
        T: Transform<BoxHandler>,
        T::Output: Handler<Request, Output = Result<Response>>,
    {
        self.map_handler(|handler| t.transform(handler).boxed())
    }

    /// Adds a middleware for the routes.
    pub fn with_handler<F>(self, f: F) -> Self
    where
        F: Handler<Next<Request, BoxHandler>, Output = Result<Response>> + Clone,
    {
        self.map_handler(|handler| handler.around(f.clone()).boxed())
    }
}

#[cfg(test)]
mod tests {
    use crate::{any, get, Resources, Route, Router, Tree};
    use viz_core::{
        async_trait, types::Params, Body, Error, Handler, HandlerExt, IntoResponse, Method,
        Request, RequestExt, Response, Result, StatusCode, Transform,
    };

    #[derive(Clone)]
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

    #[tokio::test]
    async fn router() -> anyhow::Result<()> {
        async fn index(_: Request) -> Result<Response> {
            Ok(Response::new("index".into()))
        }

        async fn all(_: Request) -> Result<Response> {
            Ok(Response::new("any".into()))
        }

        async fn not_found(_: Request) -> Result<impl IntoResponse> {
            Ok(StatusCode::NOT_FOUND)
        }

        async fn search(_: Request) -> Result<Response> {
            Ok(Response::new("search".into()))
        }

        async fn show(req: Request) -> Result<Response> {
            let ids: Vec<String> = req.params()?;
            let items = ids.into_iter().fold(String::new(), |mut s, id| {
                s.push(' ');
                s.push_str(&id);
                s
            });
            Ok(Response::new(("show".to_string() + &items).into()))
        }

        async fn create(_: Request) -> Result<Response> {
            Ok(Response::new("create".into()))
        }

        async fn update(req: Request) -> Result<Response> {
            let ids: Vec<String> = req.params()?;
            let items = ids.into_iter().fold(String::new(), |mut s, id| {
                s.push(' ');
                s.push_str(&id);
                s
            });
            Ok(Response::new(("update".to_string() + &items).into()))
        }

        async fn delete(req: Request) -> Result<Response> {
            let ids: Vec<String> = req.params()?;
            let items = ids.into_iter().fold(String::new(), |mut s, id| {
                s.push(' ');
                s.push_str(&id);
                s
            });
            Ok(Response::new(("delete".to_string() + &items).into()))
        }

        let users = Resources::default()
            .named("user")
            .index(index)
            .create(create.before(|r: Request| async { Ok(r) }))
            .show(show)
            .update(update)
            .destroy(delete)
            .map_handler(|h| {
                h.and_then(|res: Response| async {
                    let (parts, body) = res.into_parts();

                    let mut buf = bytes::BytesMut::new();
                    buf.extend(b"users: ");
                    buf.extend(hyper::body::to_bytes(body).await.map_err(Error::normal)?);

                    Ok(Response::from_parts(parts, Body::from(buf.freeze())))
                })
                .boxed()
            });

        let posts = Router::new().route("search", get(search)).resources(
            "",
            Resources::default()
                .named("post")
                .create(create)
                .show(show)
                .update(update)
                .destroy(delete)
                .map_handler(|h| {
                    h.and_then(|res: Response| async {
                        let (parts, body) = res.into_parts();

                        let mut buf = bytes::BytesMut::new();
                        buf.extend(b"posts: ");
                        buf.extend(hyper::body::to_bytes(body).await.map_err(Error::normal)?);

                        Ok(Response::from_parts(parts, Body::from(buf.freeze())))
                    })
                    .boxed()
                }),
        );

        let router = Router::new()
            // .route("", get(index))
            .get("", index)
            .resources("users", users.clone())
            .nest("posts", posts.resources(":post_id/users", users))
            .route("search", any(all))
            .route("*", Route::new().any(not_found))
            .with(Logger::new());

        let tree: Tree = router.into();

        // GET /posts
        let (req, method, path) = client(Method::GET, "/posts");
        let node = tree.find(&method, &path);
        assert!(node.is_some());
        let (h, _) = node.unwrap();
        assert_eq!(
            hyper::body::to_bytes(h.call(req).await?.into_body()).await?,
            ""
        );

        // POST /posts
        let (req, method, path) = client(Method::POST, "/posts");
        let node = tree.find(&method, &path);
        assert!(node.is_some());
        let (h, _) = node.unwrap();
        assert_eq!(
            hyper::body::to_bytes(h.call(req).await?.into_body()).await?,
            "posts: create"
        );

        // GET /posts/foo
        let (mut req, method, path) = client(Method::GET, "/posts/foo");
        let node = tree.find(&method, &path);
        assert!(node.is_some());
        let (h, params) = node.unwrap();
        req.extensions_mut().insert(Into::<Params>::into(params));
        assert_eq!(
            hyper::body::to_bytes(h.call(req).await?.into_body()).await?,
            "posts: show foo"
        );

        // PUT /posts/foo
        let (mut req, method, path) = client(Method::PUT, "/posts/foo");
        let node = tree.find(&method, &path);
        assert!(node.is_some());
        let (h, params) = node.unwrap();
        req.extensions_mut().insert(Into::<Params>::into(params));
        assert_eq!(
            hyper::body::to_bytes(h.call(req).await?.into_body()).await?,
            "posts: update foo"
        );

        // DELETE /posts/foo
        let (mut req, method, path) = client(Method::DELETE, "/posts/foo");
        let node = tree.find(&method, &path);
        assert!(node.is_some());
        let (h, params) = node.unwrap();
        req.extensions_mut().insert(Into::<Params>::into(params));
        assert_eq!(
            hyper::body::to_bytes(h.call(req).await?.into_body()).await?,
            "posts: delete foo"
        );

        // GET /posts/foo/users
        let (req, method, path) = client(Method::GET, "/posts/foo/users");
        let node = tree.find(&method, &path);
        assert!(node.is_some());
        let (h, _) = node.unwrap();
        assert_eq!(
            hyper::body::to_bytes(h.call(req).await?.into_body()).await?,
            "users: index"
        );

        // POST /posts/users
        let (req, method, path) = client(Method::POST, "/posts/foo/users");
        let node = tree.find(&method, &path);
        assert!(node.is_some());
        let (h, _) = node.unwrap();
        assert_eq!(
            hyper::body::to_bytes(h.call(req).await?.into_body()).await?,
            "users: create"
        );

        // GET /posts/foo/users/bar
        let (mut req, method, path) = client(Method::GET, "/posts/foo/users/bar");
        let node = tree.find(&method, &path);
        assert!(node.is_some());
        let (h, params) = node.unwrap();
        req.extensions_mut().insert(Into::<Params>::into(params));
        assert_eq!(
            hyper::body::to_bytes(h.call(req).await?.into_body()).await?,
            "users: show foo bar"
        );

        // PUT /posts/foo/users/bar
        let (mut req, method, path) = client(Method::PUT, "/posts/foo/users/bar");
        let node = tree.find(&method, &path);
        assert!(node.is_some());
        let (h, params) = node.unwrap();
        req.extensions_mut().insert(Into::<Params>::into(params));
        assert_eq!(
            hyper::body::to_bytes(h.call(req).await?.into_body()).await?,
            "users: update foo bar"
        );

        // DELETE /posts/foo/users/bar
        let (mut req, method, path) = client(Method::DELETE, "/posts/foo/users/bar");
        let node = tree.find(&method, &path);
        assert!(node.is_some());
        let (h, params) = node.unwrap();
        req.extensions_mut().insert(Into::<Params>::into(params));
        assert_eq!(
            hyper::body::to_bytes(h.call(req).await?.into_body()).await?,
            "users: delete foo bar"
        );

        Ok(())
    }

    fn client(method: Method, path: &str) -> (Request, Method, String) {
        (
            Request::builder()
                .method(method.to_owned())
                .uri(path.to_owned())
                .body(Body::empty())
                .unwrap(),
            method,
            path.to_string(),
        )
    }
}
