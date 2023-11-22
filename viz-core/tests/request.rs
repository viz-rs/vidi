use std::collections::{BTreeMap, HashMap};

use headers::{authorization::Bearer, Authorization, ContentType, HeaderValue};
use http::uri::Scheme;
use serde::{Deserialize, Serialize};
use viz_core::{
    // TODO: reqwest and hyper haven't used the same version of `http`.
    // header::{AUTHORIZATION, CONTENT_TYPE, COOKIE, SET_COOKIE},
    // StatusCode,
    header::CONTENT_TYPE,
    types::{self, PayloadError},
    Error,
    IncomingBody,
    IntoResponse,
    Request,
    RequestExt,
    Response,
    ResponseExt,
    Result,
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
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
    assert_eq!(
        req.incoming().unwrap_err().to_string(),
        PayloadError::Empty.to_string()
    );
    assert_eq!(
        req.incoming().unwrap_err().to_string(),
        PayloadError::Used.to_string()
    );

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
    use futures_util::stream::TryStreamExt;
    use viz::{
        middleware::{cookie, limits},
        Router,
    };
    use viz_test::http::{
        header::{AUTHORIZATION, COOKIE},
        StatusCode,
    };
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
        .post("/extract-body", |mut req: Request| async move {
            let form: types::Form<BTreeMap<String, String>> = req.extract().await?;
            Ok(Response::json(form.into_inner()))
        })
        .get("/cookies", |req: Request| async move {
            let cookies = req.cookies()?;
            let jar = cookies
                .jar()
                .lock()
                .map_err(|e| Error::Responder(e.to_string().into_response()))?;
            Ok(jar.iter().count().to_string())
        })
        .get("/cookie", |req: Request| async move {
            Ok(req.cookie("viz").unwrap().value().to_string())
        })
        .with(cookie::Config::default())
        .post("/bytes", |mut req: Request| async move {
            let data = req.bytes().await?;
            Ok(data)
        })
        .post("/bytes-with-limit", |mut req: Request| async move {
            let data = req.bytes_with(None, 4).await?;
            Ok(data)
        })
        .post("/bytes-used", |mut req: Request| async move {
            req.bytes().await?;
            let data = req.bytes().await?;
            Ok(data)
        })
        .post("/text", |mut req: Request| async move {
            let data = req.text().await?;
            Ok(Response::text(data))
        })
        .post("/json", |mut req: Request| async move {
            let data = req.json::<Page>().await?;
            Ok(Response::json(data))
        })
        .post("/form", |mut req: Request| async move {
            let data = req.form::<HashMap<String, String>>().await?;
            Ok(Response::json(data))
        })
        .post("/multipart", |mut req: Request| async move {
            let mut multipart = req.multipart().await?;
            let mut data = HashMap::new();

            while let Some(mut field) = multipart.try_next().await? {
                let buf = field.bytes().await?.to_vec();
                data.insert(field.name, String::from_utf8(buf).map_err(Error::normal)?);
            }

            Ok(Response::json(data))
        })
        .with(limits::Config::new().limits(types::Limits::new()));

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

    let mut form = BTreeMap::new();
    form.insert("password", "rs");
    form.insert("username", "viz");
    let resp = client
        .post("/extract-body")
        .form(&form)
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(
        resp.text().await.map_err(Error::normal)?,
        r#"{"password":"rs","username":"viz"}"#
    );

    let resp = client
        .get("/cookie")
        .header(COOKIE, "viz=crate")
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(resp.text().await.map_err(Error::normal)?, "crate");

    let resp = client
        .get("/cookies")
        .header(COOKIE, "auth=true;dark=false")
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(resp.text().await.map_err(Error::normal)?, "2");

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

    let resp = client
        .post("/bytes-used")
        .body("used")
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        resp.text().await.map_err(Error::normal)?,
        "payload has been used"
    );

    let resp = client
        .post("/text")
        .body("text")
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(resp.text().await.map_err(Error::normal)?, "text");

    let resp = client
        .post("/json")
        .json(&Page { p: 1 })
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(
        resp.json::<Page>().await.map_err(Error::normal)?,
        Page { p: 1 }
    );

    let mut form = HashMap::new();
    form.insert("username", "viz");
    form.insert("password", "rs");
    let resp = client
        .post("/form")
        .form(&form)
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(
        resp.json::<HashMap<String, String>>()
            .await
            .map_err(Error::normal)?,
        {
            let mut form = HashMap::new();
            form.insert("username".to_string(), "viz".to_string());
            form.insert("password".to_string(), "rs".to_string());
            form
        }
    );

    let form = viz_test::multipart::Form::new()
        .text("key3", "3")
        .text("key4", "4");
    let resp = client
        .post("/multipart")
        .multipart(form)
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(
        resp.json::<HashMap<String, String>>()
            .await
            .map_err(Error::normal)?,
        {
            let mut form = HashMap::new();
            form.insert("key3".to_string(), "3".to_string());
            form.insert("key4".to_string(), "4".to_string());
            form
        }
    );

    Ok(())
}

#[tokio::test]
async fn request_session() -> Result<()> {
    use viz::{
        middleware::{cookie, helper::CookieOptions, session},
        Router,
    };
    use viz_test::http::header::{COOKIE, SET_COOKIE};
    use viz_test::{nano_id, sessions, TestServer};

    let router = Router::new()
        .post("/session/set", |req: Request| async move {
            let counter = req.session().get::<u64>("counter")?.unwrap_or_default() + 1;
            req.session().set("counter", counter)?;
            Ok(counter.to_string())
        })
        .with(session::Config::new(
            session::Store::new(
                sessions::MemoryStorage::new(),
                nano_id::base64::<32>,
                |sid: &str| sid.len() == 32,
            ),
            CookieOptions::default(),
        ))
        .with(cookie::Config::default());

    let client = TestServer::new(router).await?;

    let resp = client
        .post("/session/set")
        .send()
        .await
        .map_err(Error::normal)?;
    let cookie = resp.headers().get(SET_COOKIE).cloned().unwrap();
    assert_eq!(resp.text().await.map_err(Error::normal)?, "1");

    let resp = client
        .post("/session/set")
        .header(COOKIE, cookie)
        .send()
        .await
        .map_err(Error::normal)?;
    assert_eq!(resp.text().await.map_err(Error::normal)?, "2");

    Ok(())
}
