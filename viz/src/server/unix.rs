use std::{future::Future, io::Result};

use tokio::net::{unix::SocketAddr, UnixListener, UnixStream};

impl super::Listener for UnixListener {
    type Io = UnixStream;
    type Addr = SocketAddr;

    fn accept(&self) -> impl Future<Output = Result<(Self::Io, Self::Addr)>> + Send {
        UnixListener::accept(self)
    }

    fn local_addr(&self) -> Result<Self::Addr> {
        UnixListener::local_addr(self)
    }
}
