use headers::{ContentType, HeaderMapExt};
use http_body_util::{BodyExt, Full};
use hyper::body::Body;
use viz_core::{OutgoingBody, Response, ResponseExt, Result, StatusCode};

#[tokio::test]
async fn response_ext() -> Result<()> {
    let resp = Response::with(Full::new("<xml/>".into()), mime::TEXT_XML.as_ref());

    assert!(resp.ok());

    let content_type = resp.headers().typed_get::<ContentType>();

    assert_eq!(
        Into::<mime::Mime>::into(content_type.unwrap()),
        mime::TEXT_XML
    );

    let body: OutgoingBody = resp.into_body();

    assert_eq!(body.size_hint().exact(), Some(b"<xml/>".len() as u64));
    assert_eq!(body.collect().await.unwrap().to_bytes().to_vec(), b"<xml/>");

    let mut resp = Response::text("");
    *resp.status_mut() = StatusCode::NOT_FOUND;

    assert!(!resp.ok());

    let content_type = resp.headers().typed_get::<ContentType>();

    assert_eq!(
        Into::<mime::Mime>::into(content_type.unwrap()),
        mime::TEXT_PLAIN_UTF_8
    );

    let mut body: OutgoingBody = resp.into_body();

    assert_eq!(body.size_hint().exact(), Some(0));
    assert!(body.frame().await.is_none());
    assert!(body.is_end_stream());

    let resp = Response::html("<html/>");

    assert!(resp.ok());

    let content_type = resp.headers().typed_get::<ContentType>();

    assert_eq!(
        Into::<mime::Mime>::into(content_type.unwrap()),
        mime::TEXT_HTML_UTF_8
    );

    let mut body: OutgoingBody = resp.into_body();

    assert_eq!(body.size_hint().exact(), Some(7));
    assert_eq!(body.frame().await.unwrap().unwrap().into_data().unwrap(), &"<html/>"[..]);
    assert!(body.is_end_stream());

    Ok(())
}
