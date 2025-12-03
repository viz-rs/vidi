//! HTTP Request test cases

use headers::HeaderValue;
use vidi_core::{
    Body, Request, RequestExt, Result,
    header::{CONTENT_LENGTH, CONTENT_TYPE},
    types::{Form, Json, Limits, PayloadError, State, StateError},
};

#[tokio::test]
async fn from_request() -> Result<()> {
    let mut req = Request::builder()
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_JSON.as_ref()),
        )
        .header(CONTENT_LENGTH, "0")
        .body(Body::Empty)?;
    req.extensions_mut().insert(Limits::default());

    let result: Result<Json<Option<String>>, PayloadError> = req.extract().await;
    assert!(result.is_err());

    let mut req = Request::builder()
        .header(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::APPLICATION_WWW_FORM_URLENCODED.as_ref()),
        )
        .header(CONTENT_LENGTH, "0")
        .body(Body::Empty)?;
    req.extensions_mut().insert(Limits::default());

    let result: Result<Form<Option<String>>, PayloadError> = req.extract().await;
    assert!(result.is_err());

    let mut req = Request::builder().body(Body::Empty)?;
    req.extensions_mut().insert(Limits::default());

    let state: Option<State<String>> = req.extract().await?;
    assert!(state.is_none());

    let mut req = Request::builder().body(Body::Empty)?;
    req.extensions_mut().insert(Limits::default());

    let result: Result<State<String>, StateError> = req.extract().await?;
    assert!(result.is_err());

    Ok(())
}
