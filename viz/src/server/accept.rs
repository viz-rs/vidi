use std::future::Future;
use std::io::Result;

pub trait Accept {
    type Conn;
    type Addr;

    fn accept(&self) -> impl Future<Output = Result<(Self::Conn, Self::Addr)>> + Send;
}
