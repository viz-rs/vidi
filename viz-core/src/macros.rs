macro_rules! tuple_impls {
    () => {
        tuple_impls!(@impl);
    };
    ($T:ident $( $U:ident )*) => {
        tuple_impls!($( $U )*);
        tuple_impls!(@impl $T $( $U )*);
    };
    // "Private" internal implementation
    (@impl $( $T:ident )*) => {
        #[crate::async_trait]
        impl<$($T,)*> FromRequest for ($($T,)*)
        where
            $($T: FromRequest + Send,)*
            $($T::Error: IntoResponse + Send,)*
        {
            type Error = Error;

            #[allow(unused, unused_mut)]
            async fn extract(req: &mut Request) -> Result<($($T,)*), Self::Error> {
                Ok(($($T::extract(req).await.map_err(IntoResponse::into_error)?,)*))
            }
        }

        #[crate::async_trait]
        impl<$($T,)* Fun, Fut, Out> FnExt<Request, ($($T,)*)> for Fun
        where
            $($T: FromRequest + Send,)*
            $($T::Error: IntoResponse + Send,)*
            Fun: Fn($($T,)*) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Result<Out>> + Send,
        {
            type Output =  Fut::Output;

            #[allow(unused, unused_mut, non_snake_case)]
            async fn call(&self, mut req: Request) -> Self::Output {
                (self)($($T::extract(&mut req).await.map_err(IntoResponse::into_error)?,)*)
                    .await
            }
        }
    };
}
