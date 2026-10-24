//! Resource

use vidi_core::{
    BoxHandler, Handler, HandlerExt, IntoResponse, Method, Next, Request, Response, Result,
    Transform,
};

use crate::Route;

/// The kind of resource path.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Kind {
    /// Collection path: `index` and `create`.
    Empty,
    /// `new` path.
    New,
    /// Member path: `show`, `update`, `destroy` (`{}_id`).
    Id,
    /// `edit` path (`{}_id/edit`).
    Edit,
    /// Custom path.
    Custom(String),
}

/// A resourceful route provides a mapping between HTTP verbs and URLs to handlers.
#[derive(Clone, Debug, Default)]
pub struct Resources {
    name: String,
    singular: bool,
    routes: Vec<(Kind, Route)>,
}

impl Resources {
    /// Sets the resource name.
    #[must_use]
    pub fn named<S>(mut self, name: S) -> Self
    where
        S: AsRef<str>,
    {
        name.as_ref().clone_into(&mut self.name);
        self
    }

    /// Marks the resource as singular (no member ID).
    #[must_use]
    pub const fn singular(mut self) -> Self {
        self.singular = true;
        self
    }

    /// Merges `route` into the existing route for `kind`, or inserts a new pair.
    fn push_route(&mut self, kind: Kind, route: Route) {
        match self
            .routes
            .iter_mut()
            .find(|(p, _)| p == &kind)
            .map(|(_, r)| r)
        {
            Some(r) => *r = r.clone().merge(route),
            None => self.routes.push((kind, route)),
        }
    }

    /// Inserts a path-route pair.
    #[must_use]
    pub fn route<S>(mut self, path: S, route: Route) -> Self
    where
        S: AsRef<str>,
    {
        let kind = Kind::Custom(path.as_ref().to_owned());
        self.push_route(kind, route);
        self
    }

    fn on<H, O>(mut self, kind: Kind, method: Method, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse,
    {
        self.push_route(kind, Route::new().on(method, handler));
        self
    }

    /// Adds the `index` handler.
    #[must_use]
    pub fn index<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse,
    {
        self.on(Kind::Empty, Method::GET, handler)
    }

    /// Adds the `new` handler.
    #[must_use]
    pub fn new<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse,
    {
        self.on(Kind::New, Method::GET, handler)
    }

    /// Adds the `create` handler.
    #[must_use]
    pub fn create<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse,
    {
        self.on(Kind::Empty, Method::POST, handler)
    }

    /// Adds the `show` handler.
    #[must_use]
    pub fn show<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse,
    {
        self.on(Kind::Id, Method::GET, handler)
    }

    /// Adds the `edit` handler.
    #[must_use]
    pub fn edit<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse,
    {
        self.on(Kind::Edit, Method::GET, handler)
    }

    /// Adds the `update` handler (`PUT`).
    #[must_use]
    pub fn update<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse,
    {
        self.on(Kind::Id, Method::PUT, handler)
    }

    /// Adds the `update` handler (`PATCH`).
    #[must_use]
    pub fn update_with_patch<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse,
    {
        self.on(Kind::Id, Method::PATCH, handler)
    }

    /// Adds the `destroy` handler.
    #[must_use]
    pub fn destroy<H, O>(self, handler: H) -> Self
    where
        H: Handler<Request, Output = Result<O>> + Clone,
        O: IntoResponse,
    {
        self.on(Kind::Id, Method::DELETE, handler)
    }

    /// Maps each handler with the given closure.
    #[must_use]
    pub fn map_handler<F>(self, f: F) -> Self
    where
        F: Fn(BoxHandler) -> BoxHandler,
    {
        Self {
            name: self.name,
            singular: self.singular,
            routes: self
                .routes
                .into_iter()
                .map(|(path, route)| (path, route.map_handler(&f)))
                .collect(),
        }
    }

    /// Wraps each handler with the given [`Transform`].
    #[must_use]
    pub fn with<T>(self, t: T) -> Self
    where
        T: Transform<BoxHandler>,
        T::Output: Handler<Request, Output = Result<Response>> + Clone,
    {
        self.map_handler(|handler| t.transform(handler).boxed())
    }

    /// Wraps each handler with the middleware.
    #[must_use]
    pub fn with_handler<H>(self, f: H) -> Self
    where
        H: Handler<Next<Request, BoxHandler>, Output = Result<Response>> + Clone,
    {
        self.map_handler(|handler| handler.around(f.clone()).boxed())
    }
}

impl IntoIterator for Resources {
    type Item = (String, Route);

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.routes
            .into_iter()
            .map(|(kind, route)| {
                (
                    match kind {
                        Kind::Empty => String::new(),
                        Kind::New => "new".to_string(),
                        Kind::Id => {
                            if self.singular {
                                String::new()
                            } else {
                                format!(":{}_id", self.name)
                            }
                        }
                        Kind::Edit => {
                            if self.singular {
                                "edit".to_string()
                            } else {
                                format!(":{}_id/edit", self.name)
                            }
                        }
                        Kind::Custom(path) => path,
                    },
                    route,
                )
            })
            .collect::<Vec<Self::Item>>()
            .into_iter()
    }
}

#[cfg(test)]
#[allow(clippy::unused_async)]
mod tests {
    use super::Kind;
    use crate::{Resources, get};
    use http_body_util::BodyExt;
    use vidi_core::{
        Handler, HandlerExt, IntoResponse, Method, Next, Request, Response, ResponseExt, Result,
        Transform, async_trait,
    };

    #[tokio::test]
    async fn resource() -> anyhow::Result<()> {
        #[derive(Clone)]
        struct Logger;

        impl Logger {
            const fn new() -> Self {
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
            H: Handler<Request>,
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
            H: Handler<Request, Output = Result<O>>,
            O: IntoResponse,
        {
            handler.call(req).await.map(IntoResponse::into_response)
        }

        async fn index(_: Request) -> Result<impl IntoResponse> {
            Ok("index")
        }

        async fn any(_: Request) -> Result<&'static str> {
            Ok("any")
        }

        async fn index_posts(_: Request) -> Result<Vec<u8>> {
            Ok(b"index posts".to_vec())
        }

        async fn create_post(_: Request) -> Result<impl IntoResponse> {
            Ok("create post")
        }

        async fn new_post(_: Request) -> Result<Response> {
            Ok(Response::text("new post"))
        }

        async fn show_post(_: Request) -> Result<Response> {
            Ok(Response::text("show post"))
        }

        async fn edit_post(_: Request) -> Result<Response> {
            Ok(Response::text("edit post"))
        }

        async fn delete_post(_: Request) -> Result<Response> {
            Ok(Response::text("delete post"))
        }

        async fn update_post(_: Request) -> Result<Response> {
            Ok(Response::text("update post"))
        }

        async fn any_posts(_: Request) -> Result<Response> {
            Ok(Response::text("any posts"))
        }

        async fn search_posts(_: Request) -> Result<Response> {
            Ok(Response::text("search posts"))
        }

        let resource = Resources::default()
            .index(index)
            .update_with_patch(any_posts);

        assert_eq!(2, resource.into_iter().count());

        let resource = Resources::default()
            .named("post")
            .route("search", get(search_posts))
            .index(index_posts.before(before))
            .new(new_post)
            .create(create_post)
            .show(show_post.after(after))
            .edit(edit_post.around(around))
            .update(update_post)
            .destroy(delete_post)
            .update_with_patch(any)
            .with(Logger::new())
            .map_handler(|handler| {
                handler
                    .before(before)
                    .after(after)
                    .around(around)
                    .with(Logger::new())
                    .boxed()
            });

        assert_eq!(5, resource.clone().into_iter().count());
        assert_eq!(
            9,
            resource
                .clone()
                .into_iter()
                .fold(0, |sum, (_, r)| sum + r.into_iter().count())
        );

        let (_, h) = resource
            .routes
            .iter()
            .find(|(p, _)| p == &Kind::Id)
            .and_then(|(_, r)| r.methods.iter().find(|(m, _)| m == Method::GET))
            .unwrap();

        let res = h.call(Request::default()).await?;
        assert_eq!(res.into_body().collect().await?.to_bytes(), "show post");

        let handler = |_| async { Ok(()) };
        let geocoder = Resources::default()
            .singular()
            .new(handler)
            .create(handler)
            .show(handler)
            .edit(handler)
            .update(handler)
            .destroy(handler);

        assert_eq!(
            6,
            geocoder
                .into_iter()
                .fold(0, |sum, (_, r)| sum + r.into_iter().count())
        );

        Ok(())
    }
}
