use crate::{handler::BoxCloneable, BoxFuture, Handler, Request, Response, Result};

pub struct BoxHandler<I = Request, O = Result<Response>>(BoxCloneable<I, O>);

impl<I, O> BoxHandler<I, O> {
    pub fn new<H>(h: H) -> Self
    where
        H: Handler<I, Output = O> + Send + Clone + 'static,
    {
        Self(Box::new(h))
    }
}

impl<I, O> Clone for BoxHandler<I, O> {
    fn clone(&self) -> Self {
        Self(self.0.clone_box())
    }
}

impl<I, O> Handler<I> for BoxHandler<I, O> {
    type Output = O;

    fn call(&self, i: I) -> BoxFuture<Self::Output> {
        self.0.call(i)
    }
}

impl<I, O> From<BoxCloneable<I, O>> for BoxHandler<I, O> {
    fn from(value: BoxCloneable<I, O>) -> Self {
        Self(value)
    }
}
