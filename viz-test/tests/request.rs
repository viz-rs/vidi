use std::collections::{BTreeMap, HashMap};

use headers::{authorization::Bearer, Authorization};
use serde::{Deserialize, Serialize};
use viz::{
    // TODO: reqwest and hyper haven't used the same version of `http`.
    // header::{AUTHORIZATION, CONTENT_TYPE, COOKIE, SET_COOKIE},
    // StatusCode,
    types::{self},
    Error,
    IntoResponse,
    Request,
    RequestExt,
    RequestLimitsExt,
    Response,
    ResponseExt,
    Result,
};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
struct Page {
    p: u8,
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
            let mut multipart = req.limited_multipart().await?;
            let mut data = HashMap::new();

            while let Some(mut field) = multipart.try_next().await? {
                let buf = field.bytes().await?.to_vec();
                data.insert(field.name, String::from_utf8(buf).map_err(Error::boxed)?);
            }

            Ok(Response::json(data))
        })
        .with(limits::Config::new().limits(types::Limits::new()));

    let client = TestServer::new(router).await?;

    let resp = client.get("/7").send().await.map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "7");

    let resp = client
        .get("/viz-rs/viz")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "viz-rs/viz");

    let resp = client
        .get("/extract-token")
        .header(AUTHORIZATION, "Bearer viz.rs")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "viz.rs");

    let mut form = BTreeMap::new();
    form.insert("password", "rs");
    form.insert("username", "viz");
    let resp = client
        .post("/extract-body")
        .form(&form)
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(
        resp.text().await.map_err(Error::boxed)?,
        r#"{"password":"rs","username":"viz"}"#
    );

    let resp = client
        .get("/cookie")
        .header(COOKIE, "viz=crate")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "crate");

    let resp = client
        .get("/cookies")
        .header(COOKIE, "auth=true;dark=false")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "2");

    let resp = client
        .post("/bytes")
        .body("bytes")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "bytes");

    let resp = client
        .post("/bytes-with-limit")
        .body("rust")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "rust");

    let resp = client
        .post("/bytes-with-limit")
        .body("crate")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        resp.text().await.map_err(Error::boxed)?,
        "payload is too large"
    );

    let resp = client
        .post("/bytes-used")
        .body("used")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        resp.text().await.map_err(Error::boxed)?,
        "payload has been used"
    );

    let resp = client
        .post("/text")
        .body("text")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "text");

    let resp = client
        .post("/json")
        .json(&Page { p: 1 })
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(
        resp.json::<Page>().await.map_err(Error::boxed)?,
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
        .map_err(Error::boxed)?;
    assert_eq!(
        resp.json::<HashMap<String, String>>()
            .await
            .map_err(Error::boxed)?,
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
        .map_err(Error::boxed)?;
    assert_eq!(
        resp.json::<HashMap<String, String>>()
            .await
            .map_err(Error::boxed)?,
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
        .map_err(Error::boxed)?;
    let cookie = resp.headers().get(SET_COOKIE).cloned().unwrap();
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "1");

    let resp = client
        .post("/session/set")
        .header(COOKIE, cookie)
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "2");

    Ok(())
}
