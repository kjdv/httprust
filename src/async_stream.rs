use futures::{Stream, Poll};
use futures::prelude::*;
use tokio::io::{AsyncRead, read_exact};


pub struct AsyncStream<T>
    where T: AsyncRead {
    reader: T,
    size: usize,
}

impl<T> AsyncStream<T>
    where T: AsyncRead {

    pub fn new(reader: T) -> AsyncStream<T> {
        const DEFAULT_SIZE: usize = 0xffff;
        AsyncStream::with_size(reader, DEFAULT_SIZE)
    }

    pub fn with_size(reader: T, max_size: usize) -> AsyncStream<T> {
        AsyncStream{
            reader: reader,
            size: max_size,
        }
    }

    fn make_buf(&self) -> Vec<u8> {
        let mut nb = Vec::with_capacity(self.size);
        nb.resize(self.size, 0);
        nb
    }
}

impl<T> Stream for AsyncStream<T>
    where T: AsyncRead {
    type Item = Vec<u8>;
    type Error = std::io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let buf = self.make_buf();
        match read_exact(&mut self.reader, buf).poll() {
            Ok(Async::Ready(v)) => Ok(Async::Ready(Some(v.1))),
            _ => Ok(Async::NotReady)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    type Chunk = Poll<Vec<u8>, std::io::Error>;

    struct FakeRead {
        input: Vec<Chunk>
    }

    impl FakeRead {
        fn new(input: Vec<Chunk>) -> FakeRead {
            FakeRead {
                input: input
            }
        }
    }

    impl Read for FakeRead {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.input.is_empty() {
                return Ok(0);
            }

            match self.input.remove(0) {
                Ok(Async::Ready(b)) => {
                    assert!(buf.len() >= b.len());
                    let len = std::cmp::min(buf.len(), b.len());
                    let b = b.as_slice();
                    buf[..len].clone_from_slice(&b);

                    Ok(len)
                },
                Ok(Async::NotReady) => Err(std::io::Error::new(std::io::ErrorKind::WouldBlock, "would block")),
                Err(e) => Err(e),
            }
        }
    }

    impl AsyncRead for FakeRead {
        fn poll_read(&mut self, _buf: &mut [u8]) -> Poll<usize, std::io::Error> {
            panic!("should not be called directly");
        }
    }

    fn make_ready(v: &str) -> Async<Option<Vec<u8>>> {
        Async::Ready(Some(Vec::from(v)))
    }

    #[test]
    fn exact_single() {
        let mut stream = AsyncStream::with_size(
            FakeRead::new(vec![
                Ok(Async::Ready(Vec::from("abcd")))
                ]), 4);

        let result = stream.poll().unwrap();
        assert_eq!(make_ready("abcd"), result);
    }
}
