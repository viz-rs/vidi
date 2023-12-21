use headers::{ContentType, HeaderValue};
use http::uri::Scheme;
use serde::{Deserialize, Serialize};
use viz_core::{
    // TODO: reqwest and hyper haven't used the same version of `http`.
    // header::{AUTHORIZATION, CONTENT_TYPE, COOKIE, SET_COOKIE},
    // StatusCode,
    header::CONTENT_TYPE,
    types::PayloadError,
    Body,
    Request,
    RequestExt,
    Result,
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Page {
    p: u8,
}

#[test]
fn request_ext() -> Result<()> {
    let mut req = Request::builder().uri("viz.rs").body(Body::Empty)?;

    assert_eq!(req.schema(), None);
    assert_eq!(req.uri(), "viz.rs");
    assert_eq!(req.path(), "");
    assert!(req.query_string().is_none());
    assert_eq!(
        req.query::<Page>().unwrap_err().to_string(),
        "url decode failed, missing field `p`"
    );
    assert!(req.header::<_, String>(CONTENT_TYPE).is_none());
    assert!(req.header_typed::<ContentType>().is_none());
    assert!(req.content_length().is_none());
    assert_eq!(
        req.incoming().unwrap_err().to_string(),
        PayloadError::Empty.to_string()
    );
    assert_eq!(
        req.incoming().unwrap_err().to_string(),
        PayloadError::Empty.to_string()
    );

    let mut req = Request::builder()
        .uri("viz.rs")
        .body(Body::Full("test".into()))?;

    assert_eq!(req.schema(), None);
    assert_eq!(req.uri(), "viz.rs");
    assert_eq!(req.path(), "");
    assert!(req.incoming().is_ok());
    assert_eq!(
        req.incoming().unwrap_err().to_string(),
        PayloadError::Used.to_string()
    );

    let mut req = Request::builder()
        .uri("https://viz.rs?p=1")
        .body(Body::Empty)?;

    req.headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static("text/plain"));

    assert_eq!(req.schema(), Some(&Scheme::try_from("https").unwrap()));
    assert_eq!(req.uri(), "https://viz.rs?p=1");
    assert_eq!(req.path(), "/");
    assert_eq!(req.query_string(), Some("p=1"));
    assert_eq!(req.query::<Page>().unwrap(), Page { p: 1 });
    assert!(req.header::<_, String>(CONTENT_TYPE).is_some());
    assert_eq!(
        req.header_typed::<ContentType>().unwrap(),
        ContentType::text()
    );
    assert_eq!(req.content_type().unwrap(), mime::TEXT_PLAIN);
    assert!(req.remote_addr().is_none());

    Ok(())
}
