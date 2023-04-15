use http_body_util::{combinators::BoxBody, BodyExt, Full};
use viz_core::{Bytes, OutgoingBody, Result};

#[tokio::test]
async fn incoming_body() -> Result<()> {
    Ok(())
}

#[tokio::test]
async fn outgoing_body() -> Result<()> {
    use viz_core::Body;

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

    let mut boxed: OutgoingBody = BoxBody::new(Full::new(Bytes::new()).map_err(Into::into)).into();
    assert!(boxed.is_end_stream());
    let size_hint = boxed.size_hint();
    assert_eq!(size_hint.lower(), 0);
    assert_eq!(size_hint.upper(), Some(0));
    assert_eq!(&format!("{boxed:?}"), r#"Boxed(BoxBody)"#);
    assert!(boxed.frame().await.is_none());

    let mut boxed: OutgoingBody =
        BoxBody::new(Full::new(Bytes::from(vec![2, 0, 4, 8])).map_err(Into::into)).into();
    assert!(!boxed.is_end_stream());
    let size_hint = boxed.size_hint();
    assert_eq!(size_hint.lower(), 4);
    assert_eq!(size_hint.upper(), Some(4));
    assert_eq!(&format!("{boxed:?}"), r#"Boxed(BoxBody)"#);
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

    let boxed: OutgoingBody = BoxBody::new(Full::new(Bytes::new()).map_err(Into::into)).into();
    assert_eq!(boxed.size_hint(), (0, Some(0)));
    let mut reader = boxed.into_async_read();
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf).await?;
    assert!(buf.is_empty());

    let mut boxed: OutgoingBody =
        BoxBody::new(Full::new(Bytes::from(vec![2, 0, 4, 8])).map_err(Into::into)).into();
    assert_eq!(boxed.size_hint(), (4, Some(4)));
    assert_eq!(boxed.next().await.unwrap().unwrap(), vec![2, 0, 4, 8]);
    assert_eq!(boxed.size_hint(), (0, Some(0)));
    assert!(boxed.next().await.is_none());

    Ok(())
}
