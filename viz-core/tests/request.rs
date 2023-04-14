use headers::{authorization::Bearer, Authorization, ContentType, HeaderValue};
use http::uri::Scheme;
use serde::Deserialize;
use viz_core::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    types, Error, IncomingBody, Request, RequestExt, Result, StatusCode,
};

#[derive(Debug, Deserialize, PartialEq)]
struct Page {
    p: u8,
}

#[test]
fn request_ext() -> Result<()> {
    let mut req = Request::builder().uri("viz.rs").body(IncomingBody::Empty)?;

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
    assert!(req.incoming().is_none());

    let mut req = Request::builder()
        .uri("https://viz.rs?p=1")
        .body(IncomingBody::Empty)?;

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

#[tokio::test]
async fn request_body() -> Result<()> {
    use viz::{middleware::limits, Router};
    use viz_test::TestServer;

    let router = Router::new()
        .get("/:id", |req: Request| async move {
            let id = req.param::<String>("id")?;
            Ok(id)
        })
        .get("/:username/:repo", |req: Request| async move {
            let (username, repo): (String, String) = req.params()?;
            Ok(format!("{username}/{repo}"))
        })
        .get("/extract-token", |mut req: Request| async move {
            let header: types::Header<Authorization<Bearer>> = req.extract().await?;
            Ok(header.into_inner().token().to_string())
        })
        .post("/bytes", |mut req: Request| async move {
            let data = req.bytes().await?;
            Ok(data)
        })
        .post("/bytes-with-limit", |mut req: Request| async move {
            let data = req.bytes_with("text", 4).await?;
            Ok(data)
        })
        .with(limits::Config::new());

    let client = TestServer::new(router).await?;

    let resp = client.get("/7").send().await.map_err(Error::normal)?;
    assert_eq!(resp.text().await.map_err(Error::normal)?, "7");

    let resp = client
        .get("/viz-rs/viz")
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(resp.text().await.map_err(Error::normal)?, "viz-rs/viz");

    let resp = client
        .get("/extract-token")
        .header(AUTHORIZATION, "Bearer viz.rs")
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(resp.text().await.map_err(Error::normal)?, "viz.rs");

    let resp = client
        .post("/bytes")
        .body("bytes")
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(resp.text().await.map_err(Error::normal)?, "bytes");

    let resp = client
        .post("/bytes-with-limit")
        .body("rust")
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.text().await.map_err(Error::normal)?, "rust");

    let resp = client
        .post("/bytes-with-limit")
        .body("crate")
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        resp.text().await.map_err(Error::normal)?,
        "payload is too large"
    );

    Ok(())
}
