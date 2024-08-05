//! JSON type test cases

use viz_core::{types::Json, IntoResponse, ResponseExt};

#[test]
fn json() {
    let data = Json::new("json");
    let resp = data.into_response();
    assert_eq!(resp.content_type(), Some(mime::APPLICATION_JSON));

    let data = Json::new("json");
    let inner = data.clone().into_inner();
    assert_eq!(inner, "json");
}
