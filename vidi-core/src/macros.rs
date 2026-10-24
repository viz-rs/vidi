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
        impl<$($T,)*> FromRequest for ($($T,)*)
        where
            $($T: FromRequest + Send,)*
            $($T::Error: IntoResponse + Send,)*
        {
            type Error = Error;

            #[allow(unused, unused_mut, clippy::unused_async_trait_impl)]
            async fn extract(req: &mut Request) -> Result<($($T,)*), Self::Error> {
                Ok(($($T::extract(req).await.map_err(IntoResponse::into_error)?,)*))
            }
        }

        impl<$($T,)* Fun, Fut, Out> FnExt<Request, ($($T,)*)> for Fun
        where
            $($T: FromRequest + Send,)*
            $($T::Error: IntoResponse + Send,)*
            Fun: Fn($($T,)*) -> Fut + Send + Sync + 'static,
            Fut: Future<Output = Result<Out>> + Send,
        {
            type Output =  Fut::Output;

            #[allow(unused, unused_mut)]
            async fn call(&self, mut req: Request) -> Self::Output {
                (self)($($T::extract(&mut req).await.map_err(IntoResponse::into_error)?,)*)
                    .await
            }
        }
    };
}

// Common deref boilerplate for newtype wrappers: `AsRef`, `Deref`, `DerefMut`.
macro_rules! impl_deref {
    ($ty:ident) => {
        impl<T> AsRef<T> for $ty<T> {
            fn as_ref(&self) -> &T {
                &self.0
            }
        }

        impl<T> Deref for $ty<T> {
            type Target = T;

            fn deref(&self) -> &T {
                &self.0
            }
        }

        impl<T> DerefMut for $ty<T> {
            fn deref_mut(&mut self) -> &mut T {
                &mut self.0
            }
        }
    };
}

// Common newtype boilerplate for extractor wrappers: `new`/`into_inner`, `Clone`,
// deref, and transparent `Debug`.
macro_rules! impl_newtype {
    ($ty:ident, $desc:literal) => {
        impl<T> $ty<T> {
            #[doc = concat!("Creates a [`", stringify!($ty), "`].")]
            #[inline]
            pub const fn new(data: T) -> Self {
                Self(data)
            }

            #[doc = concat!("Returns the wrapped value, consuming the ", $desc, ".")]
            #[inline]
            pub fn into_inner(self) -> T {
                self.0
            }
        }

        impl<T> Clone for $ty<T>
        where
            T: Clone,
        {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        impl_deref!($ty);

        impl<T> fmt::Debug for $ty<T>
        where
            T: fmt::Debug,
        {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                T::fmt(self, f)
            }
        }
    };
}

// Common error conversion: `From<$ty> for Error`.
macro_rules! impl_from_error {
    ($ty:ident) => {
        impl From<$ty> for Error {
            fn from(e: $ty) -> Self {
                e.into_error()
            }
        }
    };
}

// Common error boilerplate: respond with a 500 and convert into `Error`.
macro_rules! impl_error {
    ($ty:ident) => {
        impl IntoResponse for $ty {
            fn into_response(self) -> Response {
                (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
            }
        }

        impl_from_error!($ty);
    };
}

// Common handler-combinator boilerplate: a `{ h, f }` wrapper struct with `new`.
macro_rules! impl_handler_combinator {
    ($ty:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug)]
        pub struct $ty<H, F> {
            h: H,
            f: F,
        }

        impl<H, F> $ty<H, F> {
            #[doc = concat!("Creates a new [`", stringify!($ty), "`].")]
            #[inline]
            pub const fn new(h: H, f: F) -> Self {
                Self { h, f }
            }
        }
    };
}
