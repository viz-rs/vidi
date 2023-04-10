use headers::{ContentType, HeaderMapExt};
use http_body_util::{BodyExt, Full};
use hyper::body::Body;
use viz_core::{OutgoingBody, Response, ResponseExt, Result};

#[tokio::test]
async fn response_ext() -> Result<()> {
    let resp = Response::with(Full::new("<xml/>".into()), mime::TEXT_XML.as_ref());

    let content_type = resp.headers().typed_get::<ContentType>();

    assert_eq!(
        Into::<mime::Mime>::into(content_type.unwrap()),
        mime::TEXT_XML
    );

    let body: OutgoingBody = resp.into_body();

    assert_eq!(body.size_hint().exact(), Some(b"<xml/>".len() as u64));
    assert_eq!(body.collect().await.unwrap().to_bytes().to_vec(), b"<xml/>");

    Ok(())
}
