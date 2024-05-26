use std::{future::Future, io::Result};

use async_net::{SocketAddr, TcpListener, TcpStream};

impl crate::Listener for TcpListener {
    type Io = TcpStream;
    type Addr = SocketAddr;

    fn accept(&self) -> impl Future<Output = Result<(Self::Io, Self::Addr)>> + Send {
        Self::accept(self)
    }

    fn local_addr(&self) -> Result<Self::Addr> {
        Self::local_addr(self)
    }
}
