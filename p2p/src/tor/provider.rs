use arti_client::DataStream;
use futures::{AsyncRead, AsyncWrite};
use tokio::io::{AsyncRead as TokioAsyncRead, AsyncWrite as TokioAsyncWrite, ReadBuf};

#[derive(Debug)]
pub struct TokioTorStream {
    inner: DataStream,
}

impl From<DataStream> for TokioTorStream {
    fn from(inner: DataStream) -> Self {
        Self { inner }
    }
}

impl AsyncRead for TokioTorStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut [u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let mut read_buf = ReadBuf::new(buf);
        futures::ready!(TokioAsyncRead::poll_read(
            std::pin::Pin::new(&mut self.inner),
            cx,
            &mut read_buf
        ))?;
        std::task::Poll::Ready(Ok(read_buf.filled().len()))
    }
}

impl AsyncWrite for TokioTorStream {
    #[inline]
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        TokioAsyncWrite::poll_write(std::pin::Pin::new(&mut self.inner), cx, buf)
    }

    #[inline]
    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        TokioAsyncWrite::poll_flush(std::pin::Pin::new(&mut self.inner), cx)
    }

    #[inline]
    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        TokioAsyncWrite::poll_shutdown(std::pin::Pin::new(&mut self.inner), cx)
    }

    #[inline]
    fn poll_write_vectored(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> std::task::Poll<std::io::Result<usize>> {
        TokioAsyncWrite::poll_write_vectored(std::pin::Pin::new(&mut self.inner), cx, bufs)
    }
}
