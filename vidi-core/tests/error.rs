//! Error test cases

use std::error::Error as StdError;
use vidi_core::{Body, Error, IntoResponse, Response, StatusCode};

#[test]
fn error() {
    let e: Error = std::io::Error::last_os_error().into();
    assert!(e.is::<std::io::Error>());
    assert!(e.downcast::<std::io::Error>().is_ok());
    let e: Error = Error::boxed(std::io::Error::last_os_error());
    assert!(e.downcast_ref::<std::io::Error>().is_some());
    let boxed: Box<dyn StdError + Send + Sync> = Box::new(std::io::Error::last_os_error());
    let mut e: Error = boxed.into();
    assert!(e.downcast_mut::<std::io::Error>().is_some());

    let e: Error = (std::io::Error::other("error"), StatusCode::OK).into();
    assert_eq!("report", e.to_string());
    assert!(e.is::<std::io::Error>());
    assert!(e.downcast::<std::io::Error>().is_ok());
    let e: Error = (std::io::Error::other("error"), StatusCode::OK).into();
    assert!(e.downcast_ref::<std::io::Error>().is_some());
    let mut e: Error = (std::io::Error::other("error"), StatusCode::OK).into();
    assert!(e.downcast_mut::<std::io::Error>().is_some());

    let e = Response::new(Body::Empty).into_error();
    assert!(!e.is::<std::io::Error>());
    let e = Response::new(Body::Empty).into_error();
    assert!(e.downcast::<std::io::Error>().is_err());
    let e = Response::new(Body::Empty).into_error();
    assert!(e.downcast_ref::<std::io::Error>().is_none());
    let mut e = Response::new(Body::Empty).into_error();
    assert!(e.downcast_mut::<std::io::Error>().is_none());

    let _: Error = http::Error::from(StatusCode::from_u16(1000).unwrap_err()).into();
}
