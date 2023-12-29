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
        #[async_trait]
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

        impl<$($T,)* Fun, Fut, Out> FnExt<($($T,)*)> for Fun
        where
            $($T: FromRequest + Send,)*
            $($T::Error: IntoResponse + Send,)*
            Fun: Fn($($T,)*) -> Fut + Send + Copy + 'static,
            Fut: Future<Output = Result<Out>> + Send,
        {
            type Output =  Fut::Output;

            #[allow(unused, unused_mut, non_snake_case)]
            fn call(&self, mut req: Request) -> BoxFuture<Self::Output> {
                let this = *self;
                let fut = async move {
                    <($($T,)*)>::extract(&mut req).and_then(move |($($T,)*)| this($($T,)*)).await
                };
                Box::pin(fut)
            }
        }
    };
}
