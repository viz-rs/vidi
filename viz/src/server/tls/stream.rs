use std::{
    future::Future,
    io::Result as IoResult,
    net::SocketAddr,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::ready;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

/// TLS State
#[derive(Debug)]
enum State<A, S> {
    Handshaking(A),
    Streaming(S),
}

/// Unified TLS stream type.
#[derive(Debug)]
pub struct Stream<A, S> {
    state: State<A, S>,
    pub(crate) remote_addr: Option<SocketAddr>,
}

impl<A, S> Stream<A, S> {
    /// Creates a new TLS stream.
    pub fn new(accept: A, remote_addr: Option<SocketAddr>) -> Self {
        Self {
            state: State::Handshaking(accept),
            remote_addr,
        }
    }
}

impl<A, S> AsyncRead for Stream<A, S>
where
    A: Future<Output = IoResult<S>> + Unpin,
    S: AsyncRead + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<IoResult<()>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_read(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl<A, S> AsyncWrite for Stream<A, S>
where
    A: Future<Output = IoResult<S>> + Unpin,
    S: AsyncWrite + Unpin,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<IoResult<usize>> {
        let pin = self.get_mut();
        match pin.state {
            State::Handshaking(ref mut accept) => match ready!(Pin::new(accept).poll(cx)) {
                Ok(mut stream) => {
                    let result = Pin::new(&mut stream).poll_write(cx, buf);
                    pin.state = State::Streaming(stream);
                    result
                }
                Err(err) => Poll::Ready(Err(err)),
            },
            State::Streaming(ref mut stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        match self.state {
            State::Handshaking(_) => Poll::Ready(Ok(())),
            State::Streaming(ref mut stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}
