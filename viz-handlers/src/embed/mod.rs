//! Static files serving and embedding.

use std::{borrow::Cow, marker::PhantomData};

use http_body_util::Full;
use rust_embed::{EmbeddedFile, RustEmbed};
use viz_core::{
    async_trait,
    header::{HeaderMap, CONTENT_TYPE, ETAG, IF_NONE_MATCH},
    types::Params,
    Handler, IntoResponse, Method, Request, Response, Result, StatusCode,
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

#[async_trait]
impl<E> Handler<Request> for File<E>
where
    E: RustEmbed + Send + Sync + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        serve::<E>(&self.0, req.method(), req.headers())
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

#[async_trait]
impl<E> Handler<Request> for Dir<E>
where
    E: RustEmbed + Send + Sync + 'static,
{
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        let path = match req
            .extensions()
            .get::<Params>()
            .and_then(|params| params.first().map(|(_, v)| v))
        {
            Some(p) => p,
            None => "index.html",
        };

        serve::<E>(path, req.method(), req.headers())
    }
}

fn serve<E>(path: &str, method: &Method, headers: &HeaderMap) -> Result<Response>
where
    E: RustEmbed + Send + Sync + 'static,
{
    if method != Method::GET {
        Err(StatusCode::METHOD_NOT_ALLOWED.into_error())?;
    }

    match E::get(path) {
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
