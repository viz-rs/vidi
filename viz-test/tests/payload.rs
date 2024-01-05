use std::collections::HashMap;

use viz::{types, Error, Request, RequestLimitsExt, Response, ResponseExt, Result};

#[tokio::test]
async fn payload() -> Result<()> {
    use viz::{middleware::limits, Router};
    use viz_test::http::StatusCode;
    use viz_test::TestServer;

    let router = Router::new()
        .post("/form", |mut req: Request| async move {
            let data = req.form_with_limit::<HashMap<String, String>>().await?;
            Ok(Response::json(data))
        })
        .post("/json", |mut req: Request| async move {
            let data = req.json_with_limit::<HashMap<String, String>>().await?;
            Ok(Response::json(data))
        })
        .post("/multipart", |mut req: Request| async move {
            let _ = req.multipart_with_limit().await?;
            Ok(())
        })
        .with(
            limits::Config::new().limits(
                types::Limits::new()
                    .set("json", 1)
                    .set("form", 1)
                    .set("multipart", 1),
            ),
        );

    let client = TestServer::new(router).await?;

    let mut form = HashMap::new();
    form.insert("username", "viz");
    form.insert("password", "rs");

    // form
    let resp = client
        .post("/form")
        .json(&form)
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(
        resp.text().await.map_err(Error::boxed)?,
        "unsupported media type, `application/x-www-form-urlencoded` is required"
    );
    let resp = client
        .post("/form")
        .form(&form)
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        resp.text().await.map_err(Error::boxed)?,
        "payload is too large"
    );

    // json
    let resp = client
        .post("/json")
        .form(&form)
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(
        resp.text().await.map_err(Error::boxed)?,
        "unsupported media type, `application/javascript; charset=utf-8` is required"
    );
    let resp = client
        .post("/json")
        .json(&form)
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        resp.text().await.map_err(Error::boxed)?,
        "payload is too large"
    );

    // multipart
    let resp = client
        .post("/multipart")
        .json(&form)
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(
        resp.text().await.map_err(Error::boxed)?,
        "unsupported media type, `multipart/form-data` is required"
    );
    let form = viz_test::multipart::Form::new()
        .text("key3", "value3")
        .text("key4", "value4");
    let resp = client
        .post("/multipart")
        .multipart(form)
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(
        resp.text().await.map_err(Error::boxed)?,
        "payload is too large"
    );

    Ok(())
}
