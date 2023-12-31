use std::future::{Future, IntoFuture};
use std::io::Result;

use futures_util::{pin_mut, TryFutureExt};
use hyper_util::{
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder,
};
use tokio::io::{AsyncRead, AsyncWrite};

use crate::{BoxFuture, Responder, Tree};

mod accept;
pub use accept::Accept;

mod tcp;
pub use tcp::*;

mod unix;
pub use unix::*;

pub struct Server<L, E> {
    tree: Tree,
    listener: L,
    builder: Builder<E>,
}

impl<L, E> Server<L, E> {
    pub fn listener(&self) -> &L {
        &self.listener
    }

    pub fn builder(&mut self) -> &mut Builder<E> {
        &mut self.builder
    }
}

impl<L> Server<L, TokioExecutor> {
    pub fn new(listener: L, tree: Tree) -> Self {
        Self {
            tree,
            listener,
            builder: Builder::new(TokioExecutor::new()),
        }
    }
}

impl<L> IntoFuture for Server<L, TokioExecutor>
where
    L: Accept + Send + 'static,
    L::Conn: AsyncWrite + AsyncRead + Send + Unpin,
    L::Addr: Clone + Send + Sync + 'static,
{
    type Output = Result<()>;
    type IntoFuture = BoxFuture<Self::Output>;

    fn into_future(self) -> Self::IntoFuture {
        let Self {
            tree,
            listener,
            builder,
        } = self;
        Box::pin(async move {
            loop {
                let (stream, remote_addr) = listener.accept().await?;
                let io = TokioIo::new(stream);
                let builder = builder.clone();
                let responder = Responder::<L::Addr>::new(tree.clone(), Some(remote_addr));

                tokio::spawn(async move {
                    if let Err(_) = builder.serve_connection(io, responder).await {}
                });
            }
        })
    }
}
