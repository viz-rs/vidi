//! HTTP Response benchmarks

#![feature(test)]

extern crate test;
use test::Bencher;

use http_body_util::Full;
use serde::Serialize;
use vidi_core::{Error, Response, ResponseExt};

#[cfg(all(feature = "json", not(miri)))]
#[bench]
fn response_json_with_to_vec(b: &mut Bencher) {
    use mime;
    use vidi_core::types::PayloadError;

    #[derive(Serialize)]
    struct Message {
        message: &'static str,
    }

    b.iter(|| {
        let body = Message {
            message: "Hello, World!",
        };

        let body = serde_json::to_vec(&body).map_err(PayloadError::Json)?;
        Ok::<Response, Error>(Response::with(
            Full::from(body),
            mime::APPLICATION_JSON.as_ref(),
        ))
    });
}

#[cfg(all(feature = "json", not(miri)))]
#[bench]
fn response_json_with_map_to_vec(b: &mut Bencher) {
    use mime;
    use vidi_core::types::PayloadError;

    #[derive(Serialize)]
    struct Message {
        message: &'static str,
    }

    b.iter(|| {
        let body = Message {
            message: "Hello, World!",
        };

        serde_json::to_vec(&body)
            .map(|buf| Response::with(Full::new(buf.into()), mime::APPLICATION_JSON.as_ref()))
            .map_err(PayloadError::Json)
    });
}

#[cfg(all(feature = "json", not(miri)))]
#[bench]
fn response_json_with_to_writer(b: &mut Bencher) {
    use bytes::{BufMut, BytesMut};
    use mime;
    use vidi_core::types::PayloadError;

    #[derive(Serialize)]
    struct Message {
        message: &'static str,
    }

    b.iter(|| {
        let body = Message {
            message: "Hello, World!",
        };

        let mut buf = BytesMut::with_capacity(128).writer();
        let () = serde_json::to_writer(&mut buf, &body).map_err(PayloadError::Json)?;

        Ok::<Response, Error>(Response::with(
            Full::new(buf.into_inner().freeze()),
            mime::APPLICATION_JSON.as_ref(),
        ))
    });
}

#[cfg(all(feature = "json", not(miri)))]
#[bench]
fn response_json_with_map_to_writer(b: &mut Bencher) {
    use bytes::{BufMut, BytesMut};
    use mime;
    use vidi_core::types::PayloadError;

    #[derive(Serialize)]
    struct Message {
        message: &'static str,
    }

    b.iter(|| {
        let body = Message {
            message: "Hello, World!",
        };

        let mut buf = BytesMut::with_capacity(128).writer();
        serde_json::to_writer(&mut buf, &body)
            .map(|()| {
                Response::with(
                    Full::new(buf.into_inner().freeze()),
                    mime::APPLICATION_JSON.as_ref(),
                )
            })
            .map_err(PayloadError::Json)
    });
}
