use hyper::rt::{Read, ReadBufCursor, Write};
use pin_project_lite::pin_project;
use std::{
    io::{Error, IoSlice},
    pin::Pin,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pin_project! {
    /// A wrapping implementing hyper IO traits for a type that
    /// implements Tokio's IO traits.
    #[derive(Debug)]
    pub struct Io<T> {
        #[pin]
        inner: T,
    }
}

impl<T> Io<T> {
    /// Wrap a type implementing Tokio's IO traits.
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    /// Borrow the inner type.
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Consume this wrapper and get the inner type.
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<T> Read for Io<T>
where
    T: AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        mut buf: ReadBufCursor<'_>,
    ) -> Poll<Result<(), Error>> {
        let n = unsafe {
            let mut tempbuf = ReadBuf::uninit(buf.as_mut());
            match AsyncRead::poll_read(self.project().inner, cx, &mut tempbuf) {
                Poll::Ready(Ok(())) => tempbuf.filled().len(),
                other => return other,
            }
        };

        unsafe {
            buf.advance(n);
        }
        Poll::Ready(Ok(()))
    }
}

impl<T> Write for Io<T>
where
    T: AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        AsyncWrite::poll_write(self.project().inner, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_flush(self.project().inner, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        AsyncWrite::poll_shutdown(self.project().inner, cx)
    }

    fn is_write_vectored(&self) -> bool {
        AsyncWrite::is_write_vectored(&self.inner)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize, Error>> {
        AsyncWrite::poll_write_vectored(self.project().inner, cx, bufs)
    }
}

impl<T> AsyncRead for Io<T>
where
    T: Read,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        tempbuf: &mut ReadBuf<'_>,
    ) -> Poll<Result<(), Error>> {
        //let init = tempbuf.initialized().len();
        let filled = tempbuf.filled().len();
        let sub_filled = unsafe {
            let mut buf = hyper::rt::ReadBuf::uninit(tempbuf.unfilled_mut());

            match Read::poll_read(self.project().inner, cx, buf.unfilled()) {
                Poll::Ready(Ok(())) => buf.filled().len(),
                other => return other,
            }
        };

        let n_filled = filled + sub_filled;
        // At least sub_filled bytes had to have been initialized.
        let n_init = sub_filled;
        unsafe {
            tempbuf.assume_init(n_init);
            tempbuf.set_filled(n_filled);
        }

        Poll::Ready(Ok(()))
    }
}

impl<T> AsyncWrite for Io<T>
where
    T: Write,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        Write::poll_write(self.project().inner, cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Write::poll_flush(self.project().inner, cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        Write::poll_shutdown(self.project().inner, cx)
    }

    fn is_write_vectored(&self) -> bool {
        Write::is_write_vectored(&self.inner)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize, Error>> {
        Write::poll_write_vectored(self.project().inner, cx, bufs)
    }
}
