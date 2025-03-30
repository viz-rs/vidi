use http_body_util::{combinators::UnsyncBoxBody, BodyExt, Full};
use viz::{Body, Bytes, Error, HttpBody, Request, RequestExt, Result};

#[tokio::test]
async fn incoming_body() -> Result<()> {
    use bytes::Buf;
    use viz::{Body, Router};
    use viz_test::TestServer;

    let mut empty = Body::Empty;
    assert!(empty.is_end_stream());
    let size_hint = empty.size_hint();
    assert_eq!(size_hint.lower(), 0);
    assert_eq!(size_hint.upper(), Some(0));
    assert_eq!(&format!("{empty:?}"), "Empty");
    assert!(empty.frame().await.is_none());
    assert!(empty.frame().await.is_none());

    let router = Router::new()
        .post("/login-empty", |req: Request| async move {
            let body = req.body();
            assert!(body.is_end_stream());
            let size_hint = body.size_hint();
            assert_eq!(size_hint.lower(), 0);
            assert_eq!(size_hint.upper(), Some(0));
            Ok(())
        })
        .post("/login", |mut req: Request| async move {
            let body = req.incoming()?;
            assert!(!body.is_end_stream());
            let size_hint = body.size_hint();
            assert_eq!(size_hint.lower(), 12);
            assert_eq!(size_hint.upper(), Some(12));
            let buffered = body.collect().await.unwrap();
            let mut buf = buffered.to_bytes();
            assert_eq!(&buf.copy_to_bytes(buf.remaining())[..], b"hello world!");
            Ok(())
        });

    let client = TestServer::new(router).await?;

    let resp = client
        .post("/login-empty")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "");

    let resp = client
        .post("/login")
        .body("hello world!")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "");

    Ok(())
}

#[tokio::test]
async fn incoming_stream() -> Result<()> {
    use futures_util::{AsyncReadExt, Stream, StreamExt, TryStreamExt};
    use viz::Router;
    use viz_test::TestServer;

    let empty = Body::Empty;
    assert_eq!(Stream::size_hint(&empty), (0, Some(0)));
    let mut reader = TryStreamExt::map_err(empty, std::io::Error::other).into_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    assert!(buf.is_empty());

    let router = Router::new()
        .post("/login-empty", |mut req: Request| async move {
            let mut body = req.incoming()?;
            let size_hint = Stream::size_hint(&body);
            assert_eq!(size_hint.0, 0);
            assert_eq!(size_hint.1, Some(0));
            assert!(body.next().await.is_none());
            Ok(())
        })
        .post("/login", |mut req: Request| async move {
            let mut body = req.incoming()?;
            let size_hint = Stream::size_hint(&body);
            assert_eq!(size_hint.0, 12);
            assert_eq!(size_hint.1, Some(12));
            assert_eq!(
                body.next().await.unwrap().unwrap().to_vec(),
                b"hello world!"
            );
            assert!(body.next().await.is_none());
            Ok(())
        });

    let client = TestServer::new(router).await?;

    let resp = client
        .post("/login-empty")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "");

    let resp = client
        .post("/login")
        .body("hello world!")
        .send()
        .await
        .map_err(Error::boxed)?;
    assert_eq!(resp.text().await.map_err(Error::boxed)?, "");

    Ok(())
}

#[tokio::test]
async fn outgoing_body() -> Result<()> {
    use viz::Body;

    let mut empty = Body::<Bytes>::Empty;
    assert!(empty.is_end_stream());
    let size_hint = HttpBody::size_hint(&empty);
    assert_eq!(size_hint.lower(), 0);
    assert_eq!(size_hint.upper(), Some(0));
    assert_eq!(&format!("{empty:?}"), "Empty");
    assert!(empty.frame().await.is_none());
    assert!(empty.frame().await.is_none());

    let mut full_none = Body::from(Full::new(Bytes::new()));
    assert!(full_none.is_end_stream());
    let size_hint = full_none.size_hint();
    assert_eq!(size_hint.lower(), 0);
    assert_eq!(size_hint.upper(), Some(0));
    assert_eq!(&format!("{full_none:?}"), "Full(Full { data: None })");
    assert!(full_none.frame().await.is_none());
    assert!(full_none.frame().await.is_none());

    let mut full_some = Body::from(Full::new(Bytes::from(vec![1, 0, 2, 4])));
    assert!(!full_some.is_end_stream());
    let size_hint = full_some.size_hint();
    assert_eq!(size_hint.lower(), 4);
    assert_eq!(size_hint.upper(), Some(4));
    assert_eq!(
        &format!("{full_some:?}"),
        r#"Full(Full { data: Some(b"\x01\0\x02\x04") })"#
    );
    assert_eq!(
        full_some
            .frame()
            .await
            .unwrap()
            .unwrap()
            .into_data()
            .unwrap()
            .as_ref(),
        [1, 0, 2, 4]
    );
    assert!(full_some.frame().await.is_none());

    let mut boxed: Body = UnsyncBoxBody::new(Full::new(Bytes::new()).map_err(Into::into)).into();
    assert!(!boxed.is_end_stream());
    // boxed stream uses default size
    let size_hint = boxed.size_hint();
    assert_eq!(size_hint.lower(), 0);
    assert_eq!(size_hint.upper(), None);
    assert_eq!(&format!("{boxed:?}"), r"Boxed(SyncWrapper)");
    assert!(boxed.frame().await.is_none());

    let mut boxed: Body =
        UnsyncBoxBody::new(Full::new(Bytes::from(vec![2, 0, 4, 8])).map_err(Into::into)).into();
    assert!(!boxed.is_end_stream());
    // boxed stream uses default size
    let size_hint = boxed.size_hint();
    assert_eq!(size_hint.lower(), 0);
    assert_eq!(size_hint.upper(), None);
    assert_eq!(&format!("{boxed:?}"), r"Boxed(SyncWrapper)");
    assert_eq!(
        boxed
            .frame()
            .await
            .unwrap()
            .unwrap()
            .into_data()
            .unwrap()
            .as_ref(),
        [2, 0, 4, 8]
    );
    assert!(boxed.frame().await.is_none());

    Ok(())
}

#[tokio::test]
async fn outgoing_stream() -> Result<()> {
    use futures_util::{AsyncReadExt, Stream, StreamExt, TryStreamExt};

    let empty = Body::<Bytes>::Empty;
    assert_eq!(Stream::size_hint(&empty), (0, Some(0)));
    let mut reader = empty.into_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    assert!(buf.is_empty());

    let full_none = Body::from(Full::new(Bytes::new()));
    assert_eq!(Stream::size_hint(&full_none), (0, Some(0)));
    let mut reader = full_none.into_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    assert!(buf.is_empty());

    let mut full_some: Body = Full::new(Bytes::from(vec![1, 0, 2, 4])).into();
    assert_eq!(Stream::size_hint(&full_some), (4, Some(4)));
    assert_eq!(full_some.next().await.unwrap().unwrap(), vec![1, 0, 2, 4]);
    assert_eq!(Stream::size_hint(&full_some), (0, Some(0)));
    assert!(full_some.next().await.is_none());

    let boxed: Body = UnsyncBoxBody::new(Full::new(Bytes::new()).map_err(Into::into)).into();
    assert_eq!(Stream::size_hint(&boxed), (0, None));
    let mut reader = boxed.into_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    assert!(buf.is_empty());

    let mut boxed: Body =
        UnsyncBoxBody::new(Full::new(Bytes::from(vec![2, 0, 4, 8])).map_err(Into::into)).into();
    assert_eq!(Stream::size_hint(&boxed), (0, None));
    assert_eq!(boxed.next().await.unwrap().unwrap(), vec![2, 0, 4, 8]);
    assert_eq!(Stream::size_hint(&boxed), (0, None));
    assert!(boxed.next().await.is_none());

    Ok(())
}
