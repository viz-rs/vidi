use futures_util::{Stream, StreamExt, stream};
use headers::{ContentDisposition, ContentType, HeaderMapExt};
use http_body_util::{BodyExt, Full};
use serde::{Deserialize, Serialize};
use vidi::{
    Body, Error, HttpBody, Response, ResponseExt, Result, StatusCode,
    header::{CONTENT_DISPOSITION, CONTENT_LOCATION, LOCATION},
};

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Page {
    p: u8,
}

#[tokio::test]
async fn response_ext() -> Result<()> {
    let resp = Response::with(Full::new("<xml/>".into()), mime::TEXT_XML.as_ref());
    assert!(resp.ok());
    assert!(resp.content_length().is_none());
    let content_type = resp.headers().typed_get::<ContentType>();
    assert_eq!(
        Into::<mime::Mime>::into(content_type.unwrap()),
        mime::TEXT_XML
    );

    let body: Body = resp.into_body();
    assert_eq!(
        HttpBody::size_hint(&body).exact(),
        Some(b"<xml/>".len() as u64)
    );
    assert_eq!(
        BodyExt::collect(body).await.unwrap().to_bytes().to_vec(),
        b"<xml/>"
    );

    let mut resp = Response::text("");
    *resp.status_mut() = StatusCode::NOT_FOUND;
    assert!(!resp.ok());
    let content_type = resp.headers().typed_get::<ContentType>();
    assert_eq!(
        Into::<mime::Mime>::into(content_type.unwrap()),
        mime::TEXT_PLAIN_UTF_8
    );
    let mut body: Body = resp.into_body();
    assert_eq!(HttpBody::size_hint(&body).exact(), Some(0));
    assert!(body.frame().await.is_none());
    assert!(body.is_end_stream());

    let resp = Response::html("<html/>");
    assert!(resp.ok());
    let content_type = resp.headers().typed_get::<ContentType>();
    assert_eq!(
        Into::<mime::Mime>::into(content_type.unwrap()),
        mime::TEXT_HTML_UTF_8
    );
    let mut body: Body = resp.into_body();
    assert_eq!(HttpBody::size_hint(&body).exact(), Some(7));
    assert_eq!(
        body.frame().await.unwrap().unwrap().into_data().unwrap(),
        "<html/>"
    );
    assert!(body.is_end_stream());

    let resp = Response::json(Page { p: 255 })?;
    assert!(resp.ok());
    let content_type = resp.headers().typed_get::<ContentType>();
    assert_eq!(
        Into::<mime::Mime>::into(content_type.unwrap()),
        mime::APPLICATION_JSON
    );

    let resp = Response::stream(stream::repeat("vidi").take(2).map(Result::<_, Error>::Ok));
    assert!(resp.ok());
    let body: Body = resp.into_body();
    assert_eq!(Stream::size_hint(&body), (0, None));
    let (item, stream) = body.into_future().await;
    assert_eq!(item.unwrap().unwrap().to_vec(), b"vidi");
    let (item, stream) = stream.into_future().await;
    assert_eq!(item.unwrap().unwrap().to_vec(), b"vidi");
    let (item, _) = stream.into_future().await;
    assert!(item.is_none());

    let resp = Response::attachment("inline");
    let content_disposition = resp.headers().typed_get::<ContentDisposition>().unwrap();
    assert!(content_disposition.is_inline());

    let resp = Response::attachment("attachment");
    let content_disposition = resp.headers().typed_get::<ContentDisposition>().unwrap();
    assert!(content_disposition.is_attachment());

    let resp = Response::attachment(r#"attachment; filename="filename.jpg""#);
    let content_disposition = resp.headers().get(CONTENT_DISPOSITION).unwrap();
    assert_eq!(
        content_disposition,
        r#"attachment; filename="filename.jpg""#
    );

    let resp = Response::location("/login");
    let location = resp.headers().get(CONTENT_LOCATION).unwrap();
    assert_eq!(location, "/login");

    let resp = Response::redirect("/oauth");
    let location = resp.headers().get(LOCATION).unwrap();
    assert_eq!(location, "/oauth");

    let resp = Response::redirect_with_status("/oauth", StatusCode::TEMPORARY_REDIRECT);
    let location = resp.headers().get(LOCATION).unwrap();
    assert_eq!(location, "/oauth");
    assert_eq!(resp.status(), 307);

    let resp = Response::see_other("/oauth");
    assert_eq!(resp.status(), 303);

    let resp = Response::temporary("/oauth");
    assert_eq!(resp.status(), 307);

    let resp = Response::permanent("/oauth");
    assert_eq!(resp.status(), 308);

    Ok(())
}

#[test]
#[should_panic(expected = "not a redirection status code")]
fn response_ext_panic() {
    Response::redirect_with_status("/oauth", StatusCode::OK);
}

#[tokio::test]
async fn response_ext_with_server() -> Result<()> {
    use vidi::{Request, Router};
    use vidi_test::TestServer;

    let router = Router::new()
        .get("/", |_: Request| async move { Ok("") })
        .post("/", |_: Request| async move {
            Ok(Response::with(
                Full::new("<xml/>".into()),
                mime::TEXT_XML.as_ref(),
            ))
        })
        .get("/download", |_: Request| {
            let dir = std::env::var("CARGO_MANIFEST_DIR")
                .map(std::path::PathBuf::from)
                .unwrap();
            Response::download(dir.join("README.md"), Some("file.txt"))
        });

    let client = TestServer::new(router).await?;

    let resp = client.get("/").send().await.map_err(Error::boxed)?;
    assert_eq!(resp.content_length(), Some(0));
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "");

    let resp = client.post("/").send().await.map_err(Error::boxed)?;
    assert_eq!(resp.content_length(), Some(6));
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "<xml/>");

    let resp = client.get("/download").send().await.map_err(Error::boxed)?;
    assert_eq!(resp.content_length(), None);
    assert_eq!(
        resp.headers().get(http::header::CONTENT_DISPOSITION),
        Some(http::header::HeaderValue::from_static(
            "attachment; filename=\"file.txt\""
        ))
        .as_ref()
    );
    assert!(
        resp.text()
            .await
            .map_err(Error::boxed)?
            .starts_with("<p align=\"center\">")
    );

    Ok(())
}
