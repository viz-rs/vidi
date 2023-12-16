use http_body_util::{combinators::UnsyncBoxBody, BodyExt, Full};
use viz::{Bytes, Error, IncomingBody, OutgoingBody, Request, RequestExt, Result};

#[tokio::test]
async fn incoming_body() -> Result<()> {
    use bytes::Buf;
    use viz::{Body, Router};
    use viz_test::TestServer;

    let mut empty = IncomingBody::Empty;
    assert!(empty.is_end_stream());
    let size_hint = empty.size_hint();
    assert_eq!(size_hint.lower(), 0);
    assert_eq!(size_hint.upper(), Some(0));
    assert_eq!(&format!("{empty:?}"), "Empty");
    assert!(empty.frame().await.is_none());
    assert!(empty.frame().await.is_none());

    let mut used = IncomingBody::used();
    assert!(used.is_end_stream());
    let size_hint = used.size_hint();
    assert_eq!(size_hint.lower(), 0);
    assert_eq!(size_hint.upper(), Some(0));
    assert_eq!(&format!("{used:?}"), "Incoming(None)");
    assert!(used.frame().await.is_none());
    assert!(used.frame().await.is_none());

    let router = Router::new()
        .post("/login-empty", |mut req: Request| async move {
            let body = req.incoming_body();
            assert!(body.is_end_stream());
            let size_hint = body.size_hint();
            assert_eq!(size_hint.lower(), 0);
            assert_eq!(size_hint.upper(), Some(0));
            Ok(())
        })
        .post("/login", |mut req: Request| async move {
            let body = req.incoming_body();
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

    let empty = IncomingBody::Empty;
    assert_eq!(empty.size_hint(), (0, Some(0)));
    let mut reader =
        TryStreamExt::map_err(empty, |e| std::io::Error::new(std::io::ErrorKind::Other, e))
            .into_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    assert!(buf.is_empty());

    let used = IncomingBody::used();
    assert_eq!(used.size_hint(), (0, Some(0)));
    let mut reader =
        TryStreamExt::map_err(used, |e| std::io::Error::new(std::io::ErrorKind::Other, e))
            .into_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    assert!(buf.is_empty());

    let router = Router::new()
        .post("/login-empty", |mut req: Request| async move {
            let mut body = req.incoming_body();
            let size_hint = body.size_hint();
            assert_eq!(size_hint.0, 0);
            assert_eq!(size_hint.1, Some(0));
            assert!(body.next().await.is_none());
            Ok(())
        })
        .post("/login", |mut req: Request| async move {
            let mut body = req.incoming_body();
            let size_hint = body.size_hint();
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

    let mut empty = OutgoingBody::<Bytes>::Empty;
    assert!(empty.is_end_stream());
    let size_hint = empty.size_hint();
    assert_eq!(size_hint.lower(), 0);
    assert_eq!(size_hint.upper(), Some(0));
    assert_eq!(&format!("{empty:?}"), "Empty");
    assert!(empty.frame().await.is_none());
    assert!(empty.frame().await.is_none());

    let mut full_none = OutgoingBody::Full(Full::new(Bytes::new()));
    assert!(full_none.is_end_stream());
    let size_hint = full_none.size_hint();
    assert_eq!(size_hint.lower(), 0);
    assert_eq!(size_hint.upper(), Some(0));
    assert_eq!(&format!("{full_none:?}"), "Full(Full { data: None })");
    assert!(full_none.frame().await.is_none());
    assert!(full_none.frame().await.is_none());

    let mut full_some = OutgoingBody::<Bytes>::Full(Full::new(Bytes::from(vec![1, 0, 2, 4])));
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

    let mut boxed: OutgoingBody =
        UnsyncBoxBody::new(Full::new(Bytes::new()).map_err(Into::into)).into();
    assert!(boxed.is_end_stream());
    let size_hint = boxed.size_hint();
    assert_eq!(size_hint.lower(), 0);
    assert_eq!(size_hint.upper(), Some(0));
    assert_eq!(&format!("{boxed:?}"), r"Boxed(SyncWrapper)");
    assert!(boxed.frame().await.is_none());

    let mut boxed: OutgoingBody =
        UnsyncBoxBody::new(Full::new(Bytes::from(vec![2, 0, 4, 8])).map_err(Into::into)).into();
    assert!(!boxed.is_end_stream());
    let size_hint = boxed.size_hint();
    assert_eq!(size_hint.lower(), 4);
    assert_eq!(size_hint.upper(), Some(4));
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

    let empty = OutgoingBody::<Bytes>::Empty;
    assert_eq!(empty.size_hint(), (0, Some(0)));
    let mut reader = empty.into_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    assert!(buf.is_empty());

    let full_none = OutgoingBody::Full(Full::new(Bytes::new()));
    assert_eq!(full_none.size_hint(), (0, Some(0)));
    let mut reader = full_none.into_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    assert!(buf.is_empty());

    let mut full_some: OutgoingBody = Full::new(Bytes::from(vec![1, 0, 2, 4])).into();
    assert_eq!(full_some.size_hint(), (4, Some(4)));
    assert_eq!(full_some.next().await.unwrap().unwrap(), vec![1, 0, 2, 4]);
    assert_eq!(full_some.size_hint(), (0, Some(0)));
    assert!(full_some.next().await.is_none());

    let boxed: OutgoingBody =
        UnsyncBoxBody::new(Full::new(Bytes::new()).map_err(Into::into)).into();
    assert_eq!(boxed.size_hint(), (0, Some(0)));
    let mut reader = boxed.into_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    assert!(buf.is_empty());

    let mut boxed: OutgoingBody =
        UnsyncBoxBody::new(Full::new(Bytes::from(vec![2, 0, 4, 8])).map_err(Into::into)).into();
    assert_eq!(boxed.size_hint(), (4, Some(4)));
    assert_eq!(boxed.next().await.unwrap().unwrap(), vec![2, 0, 4, 8]);
    assert_eq!(boxed.size_hint(), (0, Some(0)));
    assert!(boxed.next().await.is_none());

    Ok(())
}
