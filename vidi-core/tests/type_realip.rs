//! `RealIp` type test cases

use vidi_core::{
    header::{HeaderValue, FORWARDED},
    types::RealIp,
    Request, RequestExt,
};

#[test]
fn realip() {
    let mut req = Request::default();
    req.headers_mut()
        .insert(RealIp::X_REAL_IP, HeaderValue::from_static("10.10.10.10"));
    assert_eq!(req.realip(), Some(RealIp("10.10.10.10".parse().unwrap())));

    let mut req = Request::default();
    req.headers_mut().insert(
        RealIp::X_FORWARDED_FOR,
        HeaderValue::from_static("10.10.10.10"),
    );
    assert_eq!(req.realip(), Some(RealIp("10.10.10.10".parse().unwrap())));

    let mut req = Request::default();
    req.headers_mut()
        .insert(FORWARDED, HeaderValue::from_static("for=10.10.10.10"));
    assert_eq!(req.realip(), Some(RealIp("10.10.10.10".parse().unwrap())));

    let req = Request::default();
    assert_eq!(req.realip(), None);

    let mut req = Request::default();
    req.extensions_mut()
        .insert("1.1.1.1:80".parse::<std::net::SocketAddr>().unwrap());
    assert_eq!(req.realip(), Some(RealIp("1.1.1.1".parse().unwrap())));
}
