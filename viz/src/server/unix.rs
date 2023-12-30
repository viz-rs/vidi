use std::future::Future;
use std::io::Result;

use tokio::net::{unix::SocketAddr, UnixListener, UnixStream};

impl super::Accept for UnixListener {
    type Conn = UnixStream;
    type Addr = SocketAddr;

    fn accept(&self) -> impl Future<Output = Result<(Self::Conn, Self::Addr)>> + Send {
        UnixListener::accept(self)
    }
}
