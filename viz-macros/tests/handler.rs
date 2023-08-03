#![allow(clippy::unused_async)]
#![allow(clippy::unnecessary_wraps)]

use viz::{async_trait, Error, FromRequest, Handler, IntoResponse, Request, Result, StatusCode};
use viz_macros::handler;

#[derive(Debug)]
struct Foo;

#[async_trait]
impl FromRequest for Foo {
    type Error = Error;

    async fn extract(_: &mut Request) -> Result<Self> {
        Ok(Foo)
    }
}

#[derive(Debug)]
struct Bar;

#[async_trait]
impl FromRequest for Bar {
    type Error = Error;

    async fn extract(_: &mut Request) -> Result<Self> {
        Ok(Bar)
    }
}

struct MyError(String);

impl From<MyError> for Error {
    fn from(MyError(err): MyError) -> Self {
        Error::Responder((StatusCode::INTERNAL_SERVER_ERROR, err).into_response())
    }
}

#[handler]
async fn a() -> impl IntoResponse {}

#[handler]
async fn b(_: Foo) -> Result<impl IntoResponse> {
    Ok(())
}

#[handler]
async fn c(_: Foo, _: Bar) -> Result<impl IntoResponse> {
    Ok(())
}

#[handler]
async fn d() {}

#[handler]
async fn e() -> StatusCode {
    StatusCode::OK
}

#[handler]
async fn f() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Hello, World!")
}

#[handler]
async fn g() {}

#[handler]
async fn h(_: Foo) -> Result<()> {
    Ok(())
}

#[handler]
async fn i(_: Foo) -> Result<StatusCode> {
    Ok(StatusCode::OK)
}

#[handler]
async fn j(_: Foo) -> Result<StatusCode> {
    Err(MyError("custom error".to_string()).into())
}

#[handler]
fn aa() -> impl IntoResponse {}

#[handler]
fn bb(_: Foo) -> Result<impl IntoResponse> {
    Ok(())
}

#[handler]
fn cc(_: Foo, _: Bar) -> Result<impl IntoResponse> {
    Ok(())
}

#[handler]
fn dd() {}

#[handler]
fn ee() -> StatusCode {
    StatusCode::OK
}

#[handler]
fn ff() -> (StatusCode, &'static str) {
    (StatusCode::OK, "Hello, World!")
}

#[handler]
fn gg() {}

#[handler]
fn hh(_: Foo) -> Result<()> {
    Ok(())
}

#[handler]
fn ii(_: Foo) -> Result<StatusCode> {
    Ok(StatusCode::OK)
}

#[tokio::test]
async fn test_handler() -> anyhow::Result<()> {
    assert!(a.call(Request::default()).await.is_ok());
    assert!(b.call(Request::default()).await.is_ok());
    assert!(c.call(Request::default()).await.is_ok());
    assert!(d.call(Request::default()).await.is_ok());
    assert!(e.call(Request::default()).await.is_ok());
    assert!(f.call(Request::default()).await.is_ok());
    assert!(g.call(Request::default()).await.is_ok());
    assert!(h.call(Request::default()).await.is_ok());
    assert!(i.call(Request::default()).await.is_ok());
    assert!(j.call(Request::default()).await.is_err());

    assert!(aa.call(Request::default()).await.is_ok());
    assert!(bb.call(Request::default()).await.is_ok());
    assert!(cc.call(Request::default()).await.is_ok());
    assert!(dd.call(Request::default()).await.is_ok());
    assert!(ee.call(Request::default()).await.is_ok());
    assert!(ff.call(Request::default()).await.is_ok());
    assert!(gg.call(Request::default()).await.is_ok());
    assert!(hh.call(Request::default()).await.is_ok());
    assert!(ii.call(Request::default()).await.is_ok());

    Ok(())
}
