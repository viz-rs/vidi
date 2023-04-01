use http::uri::Scheme;
use viz_core::{IncomingBody, Request, RequestExt, Result};

#[tokio::test]
async fn request_ext() -> Result<()> {
    let req = Request::builder()
        .uri("https://viz.rs")
        .body(IncomingBody::Empty)?;

    assert_eq!(req.schema(), Some(&Scheme::try_from("https").unwrap()));

    assert_eq!(req.path(), "/");

    assert_eq!(req.query_string(), None);

    assert_eq!(
        req.query::<u8>().unwrap_err().to_string(),
        "url decode failed, invalid type: map, expected u8"
    );

    Ok(())
}
