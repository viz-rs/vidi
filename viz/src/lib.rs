//! Viz
//!
//! Fast, robust, flexible, lightweight web framework for Rust.
//!
//! # Features
//!
//! * **Safety** `#![forbid(unsafe_code)]`
//!
//! * Lightweight
//!
//! * Simple + Flexible [`Handler`](#handler) & [`Middleware`](#middleware)
//!
//! * Handy [`Extractors`](#extractors)
//!
//! * Robust [`Routing`](#routing)
//!
//! # Hello Viz
//!
//! ```no_run
//! use std::{net::SocketAddr, sync::Arc};
//! use tokio::net::TcpListener;
//! use viz::{serve, Request, Result, Router, Tree};
//!
//! async fn index(_: Request) -> Result<&'static str> {
//!     Ok("Hello, Viz!")
//! }
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
//!     let listener = TcpListener::bind(addr).await?;
//!     println!("listening on http://{addr}");
//!
//!     let app = Router::new().get("/", index);
//!     let tree = Arc::new(Tree::from(app));
//!
//!     loop {
//!         let (stream, addr) = listener.accept().await?;
//!         let tree = tree.clone();
//!         tokio::task::spawn(serve(stream, tree, Some(addr)));
//!     }
//! }
//! ```
//!
//! More examples can be found [here](https://github.com/viz-rs/viz/tree/main/examples).
//!
//!
//! # Handler
//!
//! A simple pattern `async fn(Request) -> Result<Response>` is used to handle requests in Viz.
//!
//! ## Simple handlers
//!
//! ```
//! # use viz::{IntoResponse, Request, Response, ResponseExt, Result};
//! async fn index(_: Request) -> Result<Response> {
//!     Ok(Response::text("Hello, World!"))
//! }
//!
//! async fn about(_: Request) -> Result<&'static str> {
//!     Ok("About Me!")
//! }
//!
//! async fn not_found(_: Request) -> Result<impl IntoResponse> {
//!     Ok("Not Found!")
//! }
//! ```
//!
//! ## Implemented Handler trait
//!
//! The types can implement the [`Handler`] trait to customize handlers.
//!
//! ```
//! # use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
//! # use viz::{async_trait, Handler, IntoResponse, Request, RequestExt, Response, Result};
//! #[derive(Clone)]
//! struct MyHandler {
//!     code: Arc<AtomicUsize>,
//! }
//!
//! #[async_trait]
//! impl Handler<Request> for MyHandler {
//!     type Output = Result<Response>;  
//!
//!     async fn call(&self, req: Request) -> Self::Output {
//!         let path = req.path();
//!         let method = req.method().clone();
//!         let code = self.code.fetch_add(1, Ordering::SeqCst);
//!         Ok(format!("code = {}, method = {}, path = {}", code, method, path).into_response())
//!     }
//! }
//! ```
//!
//! ## With extractors
//!
//! Supports handler with `zero` or `one` or `more` extractors.
//!
//! Extractors must implement the [`FromRequest`] trait for extracting data from the [`Request`].
//!
//! When joining the routing system, it should first be converted to a handler using
//! [`into_handler`][IntoHandler::into_handler].
//!
//! ```
//! # use viz::{get, types::Params, IntoResponse, IntoHandler, Result, Router};
//! async fn show_todo(Params(id): Params<u64>) -> Result<impl IntoResponse> {
//!     Ok(format!("Hi, NO.{}", id))
//! }
//!
//! let app = Router::new().route("/:id", get(show_todo.into_handler()));
//! ```
//!
//! ### Why not supports handler with extractors by default?
//!
//! Viz allows more flexibility in organizing your code.
//!
//! ```
//! # use viz::{
//! #   types::Params, IntoResponse, IntoHandler, Request, RequestExt, Result,
//! #   Response, Router, FnExt,
//! # };
//! async fn show_user(mut req: Request) -> Result<Response> {
//!     let Params(id)  = req.extract::<Params<u64>>().await?;
//!     Ok(format!("post {}", id).into_response())
//! }
//!
//! async fn show_user_ext(Params(id): Params<u64>) -> Result<impl IntoResponse> {
//!     Ok(format!("Hi, NO.{}", id))
//! }
//!
//! async fn show_user_wrap(req: Request) -> Result<impl IntoResponse> {
//!     // https://github.com/rust-lang/rust/issues/48919
//!     // show_user_ext.call(req).await
//!     FnExt::call(&show_user_ext, req).await
//! }
//!
//! let app = Router::new()
//!     .get("/users/:id", show_user)
//!     .get("/users_wrap/:id", show_user_wrap)
//!     .get("/users_ext/:id", show_user_ext.into_handler());
//! ```
//!
//! ### Support process macros?
//!
//! Support, you can enable the `macros` feature, using `#[handler]`.
//!
//! But it's still recommended to use `into_handler` for conversion.
//!
//! ```ignore
//! # use viz::{handler, types::Params, IntoHandler, IntoResponse, Result, Router};
//! #[handler]
//! async fn show_user(Params(id): Params<u64>) -> Result<impl IntoResponse> {
//!     Ok(format!("Hi, NO.{}", id))
//! }
//!
//! async fn update_user(Params(id): Params<u64>) -> Result<impl IntoResponse> {
//!     Ok(format!("Updated, NO.{}", id))
//! }
//!
//! let app = Router::new()
//!     .get("/users/:id", show_user)
//!     .patch("/users/:id", update_user.into_handler());
//! ```
//!
//! ## Chaining and composing handlers
//!
//! The [`HandlerExt`] is an extension trait for [Handler]s that provides a variety of convenient
//! combinator functions.
//!
//! Likes the [`FutureExt`] and [`StreamExt`] trait.
//!
//! ```
//! # use viz::{
//! #   HandlerExt, IntoResponse, Request, Response, ResponseExt, Result, Router,
//! #   StatusCode, Method, Next, Handler
//! # };
//! async fn index(_: Request) -> Result<Response> {
//!     Ok(Response::text("hyper"))
//! }
//!
//! async fn before(req: Request) -> Result<Request> {
//!     if req.method() == Method::POST {
//!         Ok(req)
//!     } else {
//!         Err(StatusCode::METHOD_NOT_ALLOWED.into_error())
//!     }
//! }
//!
//! async fn around<H>((req, handler): Next<Request, H>) -> Result<Response>
//! where
//!     H: Handler<Request, Output = Result<Response>> + Clone,
//! {
//!     // before ...
//!     let result = handler.call(req).await;
//!     // after ...
//!     result
//! }
//!
//! async fn after(result: Result<Response>) -> Result<Response> {
//!     result.map(|mut res| {
//!         *res.status_mut() = StatusCode::NO_CONTENT;
//!         res
//!     })
//! }
//!
//! let routing = Router::new()
//!     .get("/", index.before(before).around(around).after(after));
//! ```
//!
//!
//! # Middleware
//!
//! Viz's middleware and handlers share a common [`Handler`] trait,
//! so its very easy to implement and extend the middleware.
//!
//! We can add middleware to a single handler, or to all handlers.
//!
//! We can also use [`Transform`] trait for wrapping the inner handler during construction.
//!
//! ```
//! # use std::time::Duration;
//! # use viz::{
//! #   async_trait, get, types::Params, Transform, HandlerExt, IntoResponse, IntoHandler,
//! #   Request, Response, ResponseExt, Result, Router, StatusCode, Next, Handler
//! # };
//! async fn index(_: Request) -> Result<Response> {
//!     Ok(StatusCode::OK.into_response())
//! }
//!
//! async fn not_found(_: Request) -> Result<impl IntoResponse> {
//!     Ok(StatusCode::OK)
//! }
//!
//! async fn show_user(Params(id): Params<u64>) -> Result<impl IntoResponse> {
//!     Ok(format!("post {}", id))
//! }
//!
//! // middleware fn
//! async fn around<H>((req, handler): Next<Request, H>) -> Result<Response>
//! where
//!     H: Handler<Request, Output = Result<Response>>,
//! {
//!     // before ...
//!     let result = handler.call(req).await;
//!     // after ...
//!     result
//! }
//!
//! // middleware struct
//! #[derive(Clone)]
//! struct MyMiddleware {}
//!
//! #[async_trait]
//! impl<H> Handler<Next<Request, H>> for MyMiddleware
//! where
//!     H: Handler<Request>,
//! {
//!     type Output = H::Output;
//!
//!     async fn call(&self, (i, h): Next<Request, H>) -> Self::Output {
//!         h.call(i).await
//!     }
//! }
//!
//! // A configuration for Timeout Middleware
//! struct Timeout {
//!     delay: Duration,
//! }
//!
//! impl Timeout {
//!     pub fn new(secs: u64) -> Self {
//!         Self { delay: Duration::from_secs(secs) }
//!     }
//! }
//!
//! impl<H: Clone> Transform<H> for Timeout {
//!     type Output = TimeoutMiddleware<H>;
//!
//!     fn transform(&self, h: H) -> Self::Output {
//!         TimeoutMiddleware(h, self.delay)
//!     }
//! }
//!
//! // Timeout Middleware
//! #[derive(Clone)]
//! struct TimeoutMiddleware<H>(H, Duration);
//!
//! #[async_trait]
//! impl<H> Handler<Request> for TimeoutMiddleware<H>
//! where
//!     H: Handler<Request> + Clone,
//! {
//!     type Output = H::Output;
//!
//!     async fn call(&self, req: Request) -> Self::Output {
//!         self.0.call(req).await
//!     }
//! }
//!
//! let app = Router::new()
//!     .get("/", index
//!         // handler level
//!         .around(around)
//!         .around(MyMiddleware {})
//!         .with(Timeout::new(1))
//!     )
//!     .route("/users/:id", get(
//!         show_user
//!             .into_handler()
//!             .map_into_response()
//!             // handler level
//!             .around(around)
//!             .with(Timeout::new(0))
//!         )
//!         .post(
//!             (|_| async { Ok(Response::text("update")) })
//!             // handler level
//!             .around(around)
//!             .with(Timeout::new(0))
//!         )
//!         // route level
//!         .with_handler(MyMiddleware {})
//!         .with(Timeout::new(2))
//!     )
//!     .get("/*", not_found
//!         .map_into_response()
//!         // handler level
//!         .around(around)
//!         .around(MyMiddleware {})
//!     )
//!     // router level
//!     .with_handler(around)
//!     .with_handler(MyMiddleware {})
//!     .with(Timeout::new(4));
//! ```
//!
//!
//! # Extractors
//!
//! Extracts data from the [`Request`].
//!
//! ```
//! # use std::{cmp, convert::Infallible};
//! # use viz::{async_trait, FromRequest, Request, RequestExt, Result};
//! struct Counter(u16);
//!
//! #[async_trait]
//! impl FromRequest for Counter {
//!     type Error = Infallible;
//!     async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
//!         let c = get_query_param(req.query_string());
//!         Ok(Counter(c))
//!     }
//! }
//!
//! fn get_query_param(query: Option<&str>) -> u16 {
//!    let query = query.unwrap_or("");
//!    let q = if let Some(pos) = query.find('q') {
//!        query.split_at(pos + 2).1.parse().unwrap_or(1)
//!    } else {
//!        1
//!    };
//!    cmp::min(500, cmp::max(1, q))
//! }
//! ```
//!
//!
//! # Routing
//!
//! The Viz router recognizes URLs and dispatches them to a handler.
//!
//! ## Simple routes
//!
//! ```
//! # use viz::{get, Request, Route, Router, Response, Result, IntoResponse};
//! #
//! async fn index(_: Request) -> Result<Response> {
//!     Ok(().into_response())
//! }
//!
//! let root = Router::new()
//!   .get("/", index)
//!   .route("/about", get(|_| async { Ok("about") }));
//!
//! let search = Router::new()
//!   .route("/", Route::new().get(|_| async { Ok("search") }));
//! ```
//!
//! ## CRUD, Verbs
//!
//! Adds routes with the HTTP method.
//!
//! ```
//! # use viz::{
//! #   get, Request, RequestExt, Router,
//! #   Response, ResponseExt, Result, IntoResponse, types::Params
//! # };
//! #
//! async fn index_todos(_: Request) -> Result<impl IntoResponse> {
//!     Ok(())
//! }
//!
//! async fn create_todo(_: Request) -> Result<&'static str> {
//!     Ok("created")
//! }
//!
//! async fn new_todo(_: Request) -> Result<Response> {
//!     Ok(Response::html(r#"
//!         <form method="post" action="/">
//!             <input name="todo" />
//!             <button type="submit">Create</button>
//!         </form>
//!     "#))
//! }
//!
//! async fn show_todo(mut req: Request) -> Result<Response> {
//!     let Params(id): Params<u64> = req.extract().await?;
//!     Ok(Response::text(format!("todo's id is {}", id)))
//! }
//!
//! async fn update_todo(_: Request) -> Result<()> {
//!     Ok(())
//! }
//!
//! async fn destroy_todo(_: Request) -> Result<()> {
//!     Ok(())
//! }
//!
//! async fn edit_todo(_: Request) -> Result<()> {
//!     Ok(())
//! }
//!
//! let todos = Router::new()
//!   .route("/", get(index_todos).post(create_todo))
//!   .post("/new", new_todo)
//!   .route("/:id", get(show_todo).patch(update_todo).delete(destroy_todo))
//!   .get("/:id/edit", edit_todo);
//! ```
//!
//! ## Resources
//!
//! ```
//! # use viz::{get, Request, Resources, Response, ResponseExt, Result};
//! #
//! // GET `/search`
//! async fn search_users(_: Request) -> Result<Response> {
//!     Ok(Response::json::<Vec<u64>>(vec![])?)
//! }
//!
//! // GET `/`
//! async fn index_users(_: Request) -> Result<Response> {
//!     Ok(Response::json::<Vec<u64>>(vec![])?)
//! }
//!
//! // GET `/new`
//! async fn new_user(_: Request) -> Result<&'static str> {
//!     Ok("User Form")
//! }
//!
//! // POST `/`
//! async fn create_user(_: Request) -> Result<&'static str> {
//!     Ok("Created User")
//! }
//!
//! // GET `/user_id`
//! async fn show_user(_: Request) -> Result<&'static str> {
//!     Ok("User ID 007")
//! }
//!
//! // GET `/user_id/edit`
//! async fn edit_user(_: Request) -> Result<&'static str> {
//!     Ok("Edit User Form")
//! }
//!
//! // PUT `/user_id`
//! async fn update_user(_: Request) -> Result<&'static str> {
//!     Ok("Updated User")
//! }
//!
//! // DELETE `/user_id`
//! async fn delete_user(_: Request) -> Result<&'static str> {
//!     Ok("Deleted User")
//! }
//!
//! let users = Resources::default()
//!   .named("user")
//!   .route("/search", get(search_users))
//!   .index(index_users)
//!   .new(new_user)
//!   .create(create_user)
//!   .show(show_user)
//!   .edit(edit_user)
//!   .update(update_user)
//!   .destroy(delete_user);
//! ```
//!
//! ## Nested
//!
//! ```ignore
//! # use viz::{any, IntoResponse, Request, Result, Router, StatusCode};
//! async fn not_found(_: Request) -> Result<impl IntoResponse> {
//!     Ok(StatusCode::NOT_FOUND)
//! }
//!
//! let app = Router::new()
//!   .nest("/", root)
//!   .nest("/search", search)
//!   .nest("/todos", todos.clone())
//!   .nest("/users", users.nest("todos", todos))
//!   .route("/*", any(not_found));
//! ```
//!
//! [`FutureExt`]: https://docs.rs/futures/latest/futures/future/trait.FutureExt.html
//! [`StreamExt`]: https://docs.rs/futures/latest/futures/stream/trait.StreamExt.html

#![doc(html_logo_url = "https://viz.rs/logo.svg")]
#![doc(html_favicon_url = "https://viz.rs/logo.svg")]
#![forbid(unsafe_code)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms),
        allow(dead_code, unused_assignments, unused_variables)
    )
))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[cfg(any(feature = "http1", feature = "http2"))]
mod responder;
#[cfg(any(feature = "http1", feature = "http2"))]
pub use responder::Responder;

#[cfg(any(feature = "http1", feature = "http2"))]
mod serve;
#[cfg(any(feature = "http1", feature = "http2"))]
pub use serve::{serve, serve_with_upgrades};

/// TLS
pub mod tls;
pub use viz_core::*;
pub use viz_router::*;

#[cfg(feature = "handlers")]
#[cfg_attr(docsrs, doc(cfg(feature = "handlers")))]
#[doc(inline)]
pub use viz_handlers as handlers;

#[cfg(any(feature = "http1", feature = "http2"))]
pub use hyper::server;

#[cfg(feature = "macros")]
#[cfg_attr(docsrs, doc(cfg(feature = "macros")))]
#[doc(inline)]
pub use viz_macros::handler;
