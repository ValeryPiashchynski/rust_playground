use futures::{Future, FutureExt};
use pin_project::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncRead, AsyncReadExt, ReadBuf};
use tokio::time::{Instant, Sleep};

#[tokio::main]
async fn main() -> Result<(), tokio::io::Error> {
    let mut buf = vec![0u8; 128 * 1024];
    let mut f = SlowRead::new(File::open("/dev/urandom").await?);
    let before = Instant::now();

    pin_utils::pin_mut!(f);

    f.read_exact(&mut buf).await?;
    println!("Read {} bytes in {:?}", buf.len(), before.elapsed());

    Ok(())
}

#[pin_project]
struct SlowRead<R> {
    #[pin]
    reader: R,
    #[pin]
    sleep: Sleep,
}

impl<R> SlowRead<R> {
    fn new(reader: R) -> Self {
        Self {
            reader,
            sleep: tokio::time::sleep(Default::default()),
        }
    }
}

impl<R> AsyncRead for SlowRead<R>
where
    R: AsyncRead + Unpin,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let mut this = self.project();

        match this.sleep.as_mut().poll(cx) {
            Poll::Ready(_) => {
                this.sleep.reset(Instant::now() + Duration::from_millis(25));
                this.reader.poll_read(cx, buf)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}
