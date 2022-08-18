//! Static file serving and directory listing.

use std::{
    collections::Bound,
    io::{Seek, SeekFrom},
    path::{Path, PathBuf},
    str::FromStr,
    time::SystemTime,
};
use tokio::io::AsyncReadExt;
use tokio_util::io::ReaderStream;

use viz_core::{
    async_trait,
    headers::{
        AcceptRanges, ContentLength, ContentRange, ContentType, ETag, HeaderMap, HeaderMapExt,
        IfMatch, IfModifiedSince, IfNoneMatch, IfUnmodifiedSince, LastModified, Range,
    },
    types::Params,
    Handler, IntoResponse, Method, Request, RequestExt, Response, ResponseExt, Result, StatusCode,
};

mod directory;
mod error;

use directory::Directory;
pub use error::Error;

/// Serve a single file.
#[derive(Clone, Debug)]
pub struct File {
    path: PathBuf,
}

impl File {
    /// Serve a new file by the specified path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        if !path.exists() {
            panic!("{} not found", path.to_string_lossy());
        }

        Self { path }
    }
}

#[async_trait]
impl Handler<Request> for File {
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        serve(&self.path, req.headers()).await
    }
}

/// Serve a directory.
#[derive(Clone, Debug)]
pub struct Dir {
    path: PathBuf,
    listing: bool,
    unlisted: Option<Vec<&'static str>>,
}

impl Dir {
    /// Serve a new directory by the specified path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();

        if !path.exists() {
            panic!("{} not found", path.to_string_lossy());
        }

        Self {
            path,
            listing: false,
            unlisted: None,
        }
    }

    /// Enable directory listing, `disabled` by default.
    pub fn listing(mut self) -> Self {
        self.listing = true;
        self
    }

    /// Exclude paths from the directory listing.
    pub fn unlisted(mut self, unlisted: Vec<&'static str>) -> Self {
        self.unlisted.replace(unlisted);
        self
    }
}

#[async_trait]
impl Handler<Request> for Dir {
    type Output = Result<Response>;

    async fn call(&self, req: Request) -> Self::Output {
        if req.method() != Method::GET {
            Err(Error::MethodNotAllowed)?;
        }

        let mut prev = false;
        let mut path = self.path.clone();

        if let Some(param) = req
            .extensions()
            .get::<Params>()
            .and_then(|params| params.first().map(|(_, v)| v))
        {
            let p = percent_encoding::percent_decode_str(param)
                .decode_utf8()
                .map_err(|_| Error::InvalidPath)?;
            sanitize_path(&mut path, &p)?;
            prev = true;
        }

        if !path.exists() {
            Err(StatusCode::NOT_FOUND.into_error())?;
        }

        if path.is_file() {
            return serve(&path, req.headers()).await;
        }

        let index = path.join("index.html");
        if index.exists() {
            return serve(&index, req.headers()).await;
        }

        if self.listing {
            return Directory::new(req.path(), prev, &path, &self.unlisted)
                .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR.into_error())
                .map(IntoResponse::into_response);
        }

        Ok(StatusCode::NOT_FOUND.into_response())
    }
}

fn sanitize_path<'a>(path: &'a mut PathBuf, p: &'a str) -> Result<()> {
    for seg in p.split('/') {
        if seg.starts_with("..") {
            return Err(StatusCode::NOT_FOUND.into_error());
        }
        if seg.contains('\\') {
            return Err(StatusCode::NOT_FOUND.into_error());
        }
        path.push(seg);
    }
    Ok(())
}

fn extract_etag(mtime: &SystemTime, size: u64) -> Option<ETag> {
    ETag::from_str(&format!(
        r#""{}-{}""#,
        mtime
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_millis(),
        size
    ))
    .ok()
}

#[inline]
async fn serve(path: &Path, headers: &HeaderMap) -> Result<Response> {
    let mut file = std::fs::File::open(path).map_err(Error::Io)?;
    let metadata = file
        .metadata()
        .map_err(|_| StatusCode::NOT_FOUND.into_error())?;

    let mut etag = None;
    let mut last_modified = None;
    let mut content_range = None;
    let mut max = metadata.len();

    if let Ok(modified) = metadata.modified() {
        etag = extract_etag(&modified, max);

        if matches!((headers.typed_get::<IfMatch>(), &etag), (Some(if_match), Some(etag)) if !if_match.precondition_passes(etag))
            || matches!(headers.typed_get::<IfUnmodifiedSince>(), Some(if_unmodified_since) if !if_unmodified_since.precondition_passes(modified))
        {
            Err(Error::PreconditionFailed)?;
        }

        if matches!((headers.typed_get::<IfNoneMatch>(), &etag), (Some(if_no_match), Some(etag)) if !if_no_match.precondition_passes(etag))
            || matches!(headers.typed_get::<IfModifiedSince>(), Some(if_modified_since) if !if_modified_since.is_modified(modified))
        {
            return Ok(StatusCode::NOT_MODIFIED.into_response());
        }

        last_modified.replace(LastModified::from(modified));
    }

    if let Some((start, end)) = headers
        .typed_get::<Range>()
        .and_then(|range| range.iter().next())
    {
        let start = match start {
            Bound::Included(n) => n,
            Bound::Excluded(n) => n + 1,
            Bound::Unbounded => 0,
        };
        let end = match end {
            Bound::Included(n) => n + 1,
            Bound::Excluded(n) => n,
            Bound::Unbounded => max,
        };

        if end < start || end > max {
            Err(Error::RangeUnsatisfied(max))?;
        }

        if start != 0 || end != max {
            if let Ok(range) = ContentRange::bytes(start..end, max) {
                max = end - start;
                content_range.replace(range);
                file.seek(SeekFrom::Start(start)).map_err(Error::Io)?;
            }
        }
    }

    let mut res = if content_range.is_some() {
        // max = end - start
        Response::stream(ReaderStream::new(tokio::fs::File::from_std(file).take(max)))
    } else {
        Response::stream(ReaderStream::new(tokio::fs::File::from_std(file)))
    };

    let headers = res.headers_mut();

    headers.typed_insert(AcceptRanges::bytes());
    headers.typed_insert(ContentLength(max));
    headers.typed_insert(ContentType::from(
        mime_guess::from_path(path).first_or_octet_stream(),
    ));

    if let Some(etag) = etag {
        headers.typed_insert(etag);
    }

    if let Some(last_modified) = last_modified {
        headers.typed_insert(last_modified);
    }

    if let Some(content_range) = content_range {
        headers.typed_insert(content_range);
        *res.status_mut() = StatusCode::PARTIAL_CONTENT;
    };

    Ok(res)
}
