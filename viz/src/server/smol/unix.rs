use std::{future::Future, io::Result};

use async_net::unix::{SocketAddr, UnixListener, UnixStream};

impl crate::Listener for UnixListener {
    type Io = UnixStream;
    type Addr = SocketAddr;

    fn accept(&self) -> impl Future<Output = Result<(Self::Io, Self::Addr)>> + Send {
        UnixListener::accept(self)
    }

    fn local_addr(&self) -> Result<Self::Addr> {
        UnixListener::local_addr(self)
    }
}
