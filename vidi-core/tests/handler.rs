//! HTTP Handler test cases

#![allow(clippy::diverging_sub_expression)]
#![allow(clippy::semicolon_if_nothing_returned)]
#![allow(clippy::similar_names)]
#![allow(clippy::unused_async)]
#![allow(clippy::wildcard_imports)]
#![allow(dead_code)]
#![allow(dependency_on_unit_never_type_fallback)]
#![allow(non_local_definitions)]
#![allow(unreachable_code)]
#![allow(unused_variables)]

use http_body_util::Full;
use vidi_core::handler::CatchError;
use vidi_core::*;

#[tokio::test]
async fn handler() -> Result<()> {
    trait HandlerPlus<I>: Handler<I> {
        fn catch_error2<F, E, R>(self, f: F) -> CatchError<Self, F, E, R>
        where
            Self: Sized,
        {
            CatchError::new(self, f)
        }
    }

    impl<I, T: Handler<I>> HandlerPlus<I> for T {}

    struct MyU8(u8);

    impl FromRequest for MyU8 {
        type Error = std::convert::Infallible;

        async fn extract(_: &mut Request) -> Result<Self, Self::Error> {
            Ok(Self(u8::MAX))
        }
    }

    struct MyString(String);

    impl FromRequest for MyString {
        type Error = std::convert::Infallible;

        async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
            Ok(Self(req.uri().path().to_string()))
        }
    }

    impl From<MyString> for Error {
        fn from(e: MyString) -> Self {
            Response::new(Full::from(e.0).into()).into_error()
        }
    }

    async fn it_works() {
        #[derive(Debug, thiserror::Error)]
        enum CustomError {
            #[error("not found 233")]
            NotFound,
        }

        impl From<CustomError> for Error {
            fn from(e: CustomError) -> Self {
                e.into_error()
            }
        }

        impl<T> From<CustomError> for Result<T> {
            fn from(e: CustomError) -> Self {
                Err(e.into_error())
            }
        }

        impl IntoResponse for CustomError {
            fn into_response(self) -> Response {
                Response::builder()
                    .status(http::StatusCode::NOT_FOUND)
                    .body(Full::from(self.to_string()).into())
                    .unwrap()
            }
        }

        #[derive(Debug, thiserror::Error)]
        enum CustomError2 {
            #[error("not found 377")]
            NotFound,
        }

        impl From<CustomError2> for Error {
            fn from(e: CustomError2) -> Self {
                Self::Report(Box::new(e), Box::new(CustomError::NotFound.into_response()))
            }
        }

        impl IntoResponse for CustomError2 {
            fn into_response(self) -> Response {
                Response::builder()
                    .status(http::StatusCode::NOT_FOUND)
                    .body(Full::from(self.to_string()).into())
                    .unwrap()
            }
        }

        async fn before(req: Request) -> Result<Request> {
            Ok(req)
        }

        async fn after(res: Result<Response>) -> Result<Response> {
            res
        }

        async fn a(_: Request) -> Result<Response> {
            // Err(CustomError::NotFound)?;
            // Err(CustomError2::NotFound)?;
            Ok(().into_response())
        }
        async fn b(_: Request) -> Result<Response> {
            Err(MyString("hello error".to_string()))?;
            Ok(().into_response())
        }
        async fn c(_: Request) -> Result<Response> {
            Err((
                std::io::Error::from(std::io::ErrorKind::AlreadyExists),
                (StatusCode::INTERNAL_SERVER_ERROR, "file read failed"),
            )
                .into())
        }
        async fn d(_: Request) -> Result<&'static str> {
            Ok("hello")
        }
        async fn e(_: Request) -> Result<impl IntoResponse> {
            Ok("hello")
        }
        async fn f(_: Request) -> Result<impl IntoResponse> {
            Ok("world")
        }
        async fn g(_: Request) -> Result<Vec<u8>> {
            Ok(vec![144, 233])
        }
        async fn h() -> Result<Vec<u8>> {
            Err(CustomError::NotFound)?;
            Ok(vec![144, 233])
        }
        async fn i(MyU8(a): MyU8) -> Result<impl IntoResponse> {
            Ok(vec![a, 233])
        }
        async fn j(MyU8(a): MyU8, MyU8(b): MyU8) -> Result<Vec<u8>> {
            Ok(vec![0, a, b])
        }
        async fn k(a: MyU8, b: MyU8, _: MyString) -> Result<Vec<u8>> {
            Ok(vec![0, a.0, b.0])
        }
        async fn l(a: MyU8, b: MyU8, _: MyString) -> Result<Response> {
            Ok(vec![0, a.0, b.0].into_response())
        }
        async fn m(_: MyU8, _: MyU8, _: MyString) -> Result<Response> {
            CustomError::NotFound.into()
        }

        #[derive(Clone)]
        struct MyBefore {
            name: String,
        }

        #[async_trait]
        impl<I: Send + 'static> Handler<I> for MyBefore {
            type Output = Result<I>;

            async fn call(&self, i: I) -> Self::Output {
                Ok(i)
            }
        }

        #[derive(Clone)]
        struct MyAfter {
            name: String,
        }

        #[async_trait]
        impl<O: Send + 'static> Handler<Result<O>> for MyAfter {
            type Output = Result<O>;

            async fn call(&self, o: Self::Output) -> Self::Output {
                o
            }
        }

        #[derive(Clone)]
        struct MyAround {
            name: String,
        }

        #[async_trait]
        impl<H, I, O> Handler<Next<I, H>> for MyAround
        where
            I: Send + 'static,
            H: Handler<I, Output = Result<O>>,
        {
            type Output = H::Output;

            async fn call(&self, (i, h): Next<I, H>) -> Self::Output {
                h.call(i).await
            }
        }

        const fn map(res: Response) -> Response {
            res
        }

        const fn map_err(err: Error) -> Error {
            err
        }

        async fn and_then(res: Response) -> Result<Response> {
            Ok(res)
        }

        async fn or_else(err: Error) -> Result<Response> {
            Err(err)
        }

        let aa = a
            .around(MyAround {
                name: "round 0".to_string(),
            })
            .before(before)
            .before(MyBefore {
                name: "My Before".to_string(),
            })
            .after(after)
            .after(MyAfter {
                name: "My After".to_string(),
            })
            .around(MyAround {
                name: "round 1".to_string(),
            })
            .map(map)
            .catch_error::<_, CustomError2, &'static str>(|_: CustomError2| async move {
                "Custom Error 2"
            })
            .catch_unwind(|_: Box<dyn std::any::Any + Send>| async move {
                let respond: () = panic!("Custom Error 2");
                respond
            });

        assert!(Handler::call(&aa, Request::new(Body::Empty)).await.is_ok());

        let th = MyAround {
            name: String::new(),
        };

        let rha = aa
            .map_into_response()
            .around(th.clone())
            .around(th)
            .around(MyAround {
                name: "round 2".to_string(),
            })
            .before(before)
            .map(map)
            .map_err(map_err)
            .or_else(or_else);
        let rhb = b.map_into_response();
        let rhc =
            c.map_into_response()
                .catch_error(|_: CustomError2| async move { "Custom Error 2" })
                .catch_error2::<_, std::io::Error, (StatusCode, String)>(
                    |e: std::io::Error| async move {
                        (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
                    },
                );
        let rhd = d
            .map_into_response()
            .map(map)
            .and_then(and_then)
            .or_else(or_else)
            .with(viz_core::middleware::cookie::Config::default());
        let rh_e = e.map_into_response().after(after);
        let rhf = f.map_into_response();
        let rhg = g.map_into_response();
        let rhh = h
            .into_handler()
            .map_into_response()
            .after(after)
            .before(before);
        let rhi = i.into_handler().map_into_response();
        let rhj = j.into_handler().map_into_response();
        let rhk = k.into_handler().map_into_response();
        let rhl = l.into_handler().map_into_response();
        let rhm = m.into_handler().map_into_response();

        assert!(Handler::call(&rhc, Request::default()).await.is_ok());

        assert!(rha.call(Request::default()).await.is_ok());

        assert!(Handler::call(&rha, Request::new(Body::Empty)).await.is_ok());

        assert!(rhb.call(Request::default()).await.is_err());

        let brha: BoxHandler<_, _> = rha.boxed();
        let brhb: BoxHandler<_, _> = Box::new(rhb)
            .around(MyAround {
                name: "MyRound 3".to_string(),
            })
            .boxed();
        let brhc: BoxHandler<_, _> = rhc.boxed();
        let brhd: BoxHandler<_, _> = rhd.boxed();
        let brhe: BoxHandler<_, _> = rh_e.boxed();
        let brhf: BoxHandler<_, _> = rhf.boxed();
        let brhg: BoxHandler<_, _> = rhg.boxed();
        let brhh: BoxHandler<_, _> = rhh.boxed();
        let brhi: BoxHandler<_, _> = rhi.boxed();
        let brhj: BoxHandler<_, _> = rhj.boxed();
        let brhk: BoxHandler<_, _> = rhk.boxed();
        let brhl: BoxHandler<_, _> = rhl.boxed();
        let brhm: BoxHandler<_, _> = rhm.boxed();

        let v: Vec<BoxHandler<_, _>> = vec![
            brha, brhb, brhc, brhd, brhe, brhf, brhg, brhh, brhi, brhj, brhk, brhl, brhm,
        ];

        #[allow(clippy::redundant_clone)]
        let y = v.clone();

        assert!(!y.is_empty());

        assert!(!v.is_empty());
    }

    it_works().await;

    Ok(())
}
