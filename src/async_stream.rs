use futures::{Stream, Poll};
use futures::prelude::*;
use tokio::io::{AsyncRead, read_exact, ReadExact};


pub struct AsyncStream<T>
    where T: AsyncRead {
    size: usize,
    reader: ReadExact<T, Vec<u8>>,
}

impl<T> AsyncStream<T>
    where T: AsyncRead {

    pub fn new(reader: T) -> AsyncStream<T> {
        const DEFAULT_SIZE: usize = 0xffff;
        AsyncStream::with_size(reader, DEFAULT_SIZE)
    }

    pub fn with_size(reader: T, max_size: usize) -> AsyncStream<T> {
        AsyncStream{
            size: max_size,
            reader: read_exact(reader, Self::make_buf(max_size)),
        }
    }

    fn make_buf(size: usize) -> Vec<u8> {
        let mut nb = Vec::with_capacity(size);
        nb.resize(size, 0);
        nb
    }
}

impl<T> Stream for AsyncStream<T>
    where T: AsyncRead {
    type Item = Vec<u8>;
    type Error = std::io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        match self.reader.poll() {
            Ok(Async::Ready(v)) => Ok(Async::Ready(Some(v.1))),
            Ok(Async::NotReady) => Ok(Async::NotReady),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    type Chunk = Result<Vec<u8>, std::io::Error>;

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
                Ok(b) => {
                    let len = std::cmp::min(buf.len(), b.len());
                    let b = b.as_slice();
                    buf[..len].clone_from_slice(&b);

                    Ok(len)
                },
                Err(e) => Err(e),
            }
        }
    }

    impl AsyncRead for FakeRead {}

    fn wouldblock() -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::WouldBlock, "would block")
    }

    fn make_ready(v: &str) -> Async<Option<Vec<u8>>> {
        Async::Ready(Some(Vec::from(v)))
    }

    #[test]
    fn exact_single() {
        let mut stream = AsyncStream::with_size(
            FakeRead::new(vec![
                Ok(Vec::from("abcd"))
                ]), 4);

        let result = stream.poll().unwrap();
        assert_eq!(make_ready("abcd"), result);
    }

    #[test]
    fn underflow() {
        let mut stream = AsyncStream::with_size(
            FakeRead::new(vec![
                Ok(Vec::from("abc".as_bytes())),
                Err(wouldblock()),
                Ok(Vec::from("d".as_bytes())),
                ]), 4);

        let result = stream.poll().unwrap();
        assert!(!result.is_ready());

        let result = stream.poll().unwrap();
        assert_eq!(make_ready("abcd"), result);
    }
}
