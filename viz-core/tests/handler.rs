#![allow(dead_code)]
#![allow(clippy::unused_async)]
#![allow(clippy::similar_names)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::wildcard_imports)]

use std::marker::PhantomData;
use viz_core::*;

#[tokio::test]
async fn main() -> Result<()> {
    pub struct CatchError<H, F, R, E> {
        h: H,
        f: F,
        r: PhantomData<R>,
        e: PhantomData<E>,
    }

    impl<H: Clone, F: Clone, R, E> Clone for CatchError<H, F, R, E> {
        fn clone(&self) -> Self {
            Self {
                h: self.h.clone(),
                f: self.f.clone(),
                r: PhantomData,
                e: PhantomData,
            }
        }
    }

    impl<H, F, R, E> CatchError<H, F, R, E> {
        #[inline]
        pub(crate) fn new(h: H, f: F) -> Self {
            Self {
                h,
                f,
                r: PhantomData,
                e: PhantomData,
            }
        }
    }

    #[async_trait]
    impl<H, F, I, O, R, E> Handler<I> for CatchError<H, F, R, E>
    where
        I: Send + 'static,
        O: IntoResponse + Send,
        H: Handler<I, Output = Result<O>> + Clone,
        F: Handler<E, Output = R> + Clone,
        R: IntoResponse + Send + Sync + 'static,
        E: std::error::Error + Send + Sync + 'static,
    {
        type Output = Result<Response>;

        async fn call(&self, i: I) -> Self::Output {
            match self.h.call(i).await {
                Ok(r) => Ok(r.into_response()),
                Err(e) if e.is::<E>() => Ok(self
                    .f
                    .call(e.downcast::<E>().unwrap())
                    .await
                    .into_response()),
                Err(e) => Err(e),
            }
        }
    }

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

    #[async_trait]
    impl FromRequest for MyU8 {
        type Error = std::convert::Infallible;

        async fn extract(_req: &mut Request) -> Result<Self, Self::Error> {
            Ok(MyU8(u8::MAX))
        }
    }

    struct MyString(String);

    #[async_trait]
    impl FromRequest for MyString {
        type Error = std::convert::Infallible;

        async fn extract(req: &mut Request) -> Result<Self, Self::Error> {
            Ok(MyString(req.uri().path().to_string()))
        }
    }

    impl From<MyString> for Error {
        fn from(e: MyString) -> Self {
            Error::Responder(Response::new(e.0.into()))
        }
    }

    async fn it_works() -> Result<()> {
        #[derive(thiserror::Error, Debug)]
        enum CustomError {
            #[error("not found 233")]
            NotFound,
        }

        impl From<CustomError> for Error {
            fn from(e: CustomError) -> Self {
                Error::Responder(e.into_response())
            }
        }

        impl<T> From<CustomError> for Result<T> {
            fn from(e: CustomError) -> Self {
                Err(Error::Responder(e.into_response()))
            }
        }

        impl IntoResponse for CustomError {
            fn into_response(self) -> Response {
                Response::builder()
                    .status(http::StatusCode::NOT_FOUND)
                    .body(self.to_string().into())
                    .unwrap()
            }
        }

        #[derive(thiserror::Error, Debug)]
        enum CustomError2 {
            #[error("not found 377")]
            NotFound,
        }

        impl From<CustomError2> for Error {
            fn from(e: CustomError2) -> Self {
                // Error::Responder(e.into_response())
                Error::Report(Box::new(e), CustomError::NotFound.into_response())
            }
        }

        impl IntoResponse for CustomError2 {
            fn into_response(self) -> Response {
                Response::builder()
                    .status(http::StatusCode::NOT_FOUND)
                    .body(self.to_string().into())
                    .unwrap()
            }
        }

        async fn before(req: Request) -> Result<Request> {
            Ok(req)
        }

        async fn after(res: Result<Response>) -> Result<Response> {
            res
        }

        async fn a(_req: Request) -> Result<Response> {
            // Err(CustomError::NotFound.into())

            Err(CustomError2::NotFound)?;
            // Err(CustomError::NotFound)?;
            Ok(().into_response())

            // Err(CustomError2::NotFound.into())
        }
        async fn b(_req: Request) -> Result<Response> {
            // Err("hello error".into())

            Err(MyString("hello error".to_string()))?;
            Ok(().into_response())
        }
        async fn c(_req: Request) -> Result<Response> {
            // Ok(Response::new("hello".into()))
            // Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "file read failed").into())

            // Err(std::io::Error::new(
            //     std::io::ErrorKind::AlreadyExists,
            //     "file read failed",
            // ))?;
            // Ok(().into_response())

            Err((
                std::io::Error::from(std::io::ErrorKind::AlreadyExists),
                (StatusCode::INTERNAL_SERVER_ERROR, "file read failed"),
            )
                .into())
        }
        async fn d(_req: Request) -> Result<&'static str> {
            Ok("hello")
        }
        async fn e(_req: Request) -> Result<impl IntoResponse> {
            Ok("hello")
        }
        async fn f(_req: Request) -> Result<impl IntoResponse> {
            Ok("world")
        }
        async fn g(_req: Request) -> Result<Vec<u8>> {
            Ok(vec![144, 233])
        }
        // async fn h() -> Vec<u8> {
        //     vec![144, 233]
        // }
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
        async fn m(_a: MyU8, _b: MyU8, _c: MyString) -> Result<Response> {
            // Err(CustomError::NotFound)?;
            // Ok(().into_response())
            CustomError::NotFound.into()
        }

        // dbg!(FnExt::call(&h, Request::new(Body::empty())).await);

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

        // #[async_trait]
        // impl<O: Send + 'static> Handler<O> for MyAfter {
        //     type Output = O;

        //     async fn call(&self, o: O) -> Self::Output {
        //         dbg!(&self.name);
        //         o
        //     }
        // }
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

        async fn map(res: Response) -> Response {
            res
        }

        async fn map_err(err: Error) -> Error {
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
            .catch_error(|_: CustomError2| async move { "Custom Error 2" })
            .catch_unwind(
                |_: Box<dyn std::any::Any + Send>| async move { panic!("Custom Error 2") },
            );

        assert!(Handler::call(&aa, Request::new(Body::empty()))
            .await
            .is_ok());

        let th = MyAround {
            name: String::new(),
        };

        // let rha = Responder::new(a);
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
        let rhc = c
            .map_into_response()
            .catch_error(|_: CustomError2| async move { "Custom Error 2" })
            .catch_error2(|e: std::io::Error| async move {
                (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
            });
        #[cfg(any(feature = "cookie-signed", feature = "cookie-private"))]
        let cookie_config =
            viz_core::middleware::cookie::Config::new(viz_core::types::CookieKey::generate());
        #[cfg(not(any(feature = "cookie-signed", feature = "cookie-private")))]
        let cookie_config = viz_core::middleware::cookie::Config::new();
        let rhd = d
            .map_into_response()
            .map(map)
            .and_then(and_then)
            .or_else(or_else)
            .with(cookie_config);
        let rhe = e.map_into_response().after(after);
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

        assert!(Handler::call(&rha, Request::new(Body::empty()))
            .await
            .is_ok());

        assert!(rhb.call(Request::default()).await.is_err());
        // dbg!(rhc.call(Request::default()).await);
        // dbg!(rhd.call(Request::default()).await);
        // dbg!(rhe.call(Request::default()).await);
        // dbg!(rhf.call(Request::default()).await);
        // dbg!(rhg.call(Request::default()).await);
        // dbg!(rhh.call(Request::default()).await);
        // dbg!(Handler::call(&rhh, Request::default()).await);
        // dbg!(rhi.call(Request::default()).await);
        // dbg!(rhj.call(Request::default()).await);

        // dbg!(Handler::call(&rhk, Request::default()).await);
        // dbg!(Handler::call(&rhl, Request::default()).await);
        // dbg!(Handler::call(&rhm, Request::default()).await);
        // dbg!(Handler::call(&rhi, Request::default()).await);

        let brha: BoxHandler = rha.boxed();
        let brhb: BoxHandler = Box::new(rhb)
            .around(MyAround {
                name: "MyRound 3".to_string(),
            })
            .boxed();
        let brhc: BoxHandler = Box::new(rhc);
        let brhd: BoxHandler = Box::new(rhd);
        let brhe: BoxHandler = rhe.boxed();
        let brhf: BoxHandler = Box::new(rhf);
        let brhg: BoxHandler = Box::new(rhg);
        let brhh: BoxHandler = Box::new(rhh);
        let brhi: BoxHandler = Box::new(rhi);
        let brhj: BoxHandler = Box::new(rhj);
        let brhk: BoxHandler = rhk.boxed();
        let brhl: BoxHandler = Box::new(rhl);
        let brhm: BoxHandler = rhm.boxed();

        let v: Vec<BoxHandler> = vec![
            brha, brhb, brhc, brhd, brhe, brhf, brhg, brhh, brhi, brhj, brhk, brhl, brhm,
        ];

        let y = v.clone();

        assert!(!y.is_empty());

        Ok(())
    }

    it_works().await
}
