//! Static files serving and embedding.

use std::{borrow::Cow, marker::PhantomData};

use http_body_util::Full;
use rust_embed::{EmbeddedFile, RustEmbed};
use viz_core::{
    header::{CONTENT_TYPE, ETAG, IF_NONE_MATCH},
    BoxFuture, Handler, IntoResponse, Method, Request, RequestExt, Response, Result, StatusCode,
};

/// Serve a single embedded file.
#[derive(Debug)]
pub struct File<E>(Cow<'static, str>, PhantomData<E>);

impl<E> Clone for File<E> {
    fn clone(&self) -> Self {
        Self(self.0.clone(), PhantomData)
    }
}

impl<E> File<E> {
    /// Serve a new file by the specified path.
    #[must_use]
    pub fn new(path: &'static str) -> Self {
        Self(path.into(), PhantomData)
    }
}

impl<E> Handler<Request> for File<E>
where
    E: RustEmbed + Send + Sync + 'static,
{
    type Output = Result<Response>;

    fn call(&self, req: Request) -> BoxFuture<Self::Output> {
        Box::pin(serve::<E>(self.0.to_string(), req))
    }
}

/// Serve a embedded directory.
#[derive(Debug)]
pub struct Dir<E>(PhantomData<E>);

impl<E> Clone for Dir<E> {
    fn clone(&self) -> Self {
        Self(PhantomData)
    }
}

impl<E> Default for Dir<E> {
    fn default() -> Self {
        Self(PhantomData)
    }
}

impl<E> Handler<Request> for Dir<E>
where
    E: RustEmbed + Send + Sync + 'static,
{
    type Output = Result<Response>;

    fn call(&self, req: Request) -> BoxFuture<Self::Output> {
        let path = match req.route_info().params.first().map(|(_, v)| v) {
            Some(p) => p,
            None => "index.html",
        }
        .to_string();

        Box::pin(serve::<E>(path, req))
    }
}

async fn serve<E>(path: String, req: Request) -> Result<Response>
where
    E: RustEmbed + Send + Sync + 'static,
{
    let method = req.method();
    let headers = req.headers();

    if method != Method::GET {
        Err(StatusCode::METHOD_NOT_ALLOWED.into_error())?;
    }

    match E::get(&path) {
        Some(EmbeddedFile { data, metadata }) => {
            let hash = hex::encode(metadata.sha256_hash());

            if headers
                .get(IF_NONE_MATCH)
                .map_or(false, |etag| etag.to_str().unwrap_or("000000").eq(&hash))
            {
                Err(StatusCode::NOT_MODIFIED.into_error())?;
            }

            Response::builder()
                .header(
                    CONTENT_TYPE,
                    mime_guess::from_path(path).first_or_octet_stream().as_ref(),
                )
                .header(ETAG, hash)
                .body(Full::from(data).into())
                .map_err(Into::into)
        }
        None => Err(StatusCode::NOT_FOUND.into_error()),
    }
}
