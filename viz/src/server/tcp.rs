use std::future::Future;
use std::io::Result;
use std::net::SocketAddr;

use tokio::net::{TcpListener, TcpStream};

impl super::Accept for TcpListener {
    type Conn = TcpStream;
    type Addr = SocketAddr;

    fn accept(&self) -> impl Future<Output = Result<(Self::Conn, Self::Addr)>> + Send {
        TcpListener::accept(self)
    }
}
