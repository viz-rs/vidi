use headers::{ContentLength, ContentType, HeaderMapExt};
use viz_core::{Error, IntoResponse, Response, StatusCode};

#[test]
fn into_response() {
    let resp = None::<()>.into_response();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    let content_type = resp.headers().typed_get::<ContentType>();
    assert!(content_type.is_none());
    let content_length = resp.headers().typed_get::<ContentLength>();
    assert!(content_length.is_none());

    let resp = Some("rust").into_response();
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp.headers().typed_get::<ContentType>().unwrap();
    assert_eq!(
        Into::<mime::Mime>::into(content_type),
        mime::TEXT_PLAIN_UTF_8
    );
    let content_length = resp.headers().typed_get::<ContentLength>().unwrap();
    assert_eq!(content_length.0, 4);

    let resp = Ok::<_, Error>("rust").into_response();
    assert_eq!(resp.status(), StatusCode::OK);
    let content_type = resp.headers().typed_get::<ContentType>().unwrap();
    assert_eq!(
        Into::<mime::Mime>::into(content_type),
        mime::TEXT_PLAIN_UTF_8
    );
    let content_length = resp.headers().typed_get::<ContentLength>().unwrap();
    assert_eq!(content_length.0, 4);

    let resp = std::io::Error::new(std::io::ErrorKind::Other, "rust").into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let content_length = resp.headers().typed_get::<ContentLength>().unwrap();
    assert_eq!(content_length.0, 4);

    let resp = Error::from(std::io::Error::new(std::io::ErrorKind::Other, "rust")).into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let content_length = resp.headers().typed_get::<ContentLength>().unwrap();
    assert_eq!(content_length.0, 4);

    let resp = Err::<Response, Error>(Error::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        "rust",
    )))
    .into_response();
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let content_length = resp.headers().typed_get::<ContentLength>().unwrap();
    assert_eq!(content_length.0, 4);

    let resp = &[].into_response();
    assert_eq!(resp.status(), StatusCode::OK);
    let content_length = resp.headers().typed_get::<ContentLength>().unwrap();
    assert_eq!(content_length.0, 0);

    let resp = Error::Responder("rust".into_response()).into_response();
    assert_eq!(resp.status(), StatusCode::OK);
    let content_length = resp.headers().typed_get::<ContentLength>().unwrap();
    assert_eq!(content_length.0, 4);
}
