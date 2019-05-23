use futures::{Stream, Poll};
use futures::prelude::*;
use tokio::io::AsyncRead;


pub struct AsyncStream<T>
    where T: AsyncRead {
    reader: T,
    buf: Vec<u8>,
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
            buf: Self::new_buf(max_size),
        }
    }

    fn size(&self) -> usize {
        self.buf.capacity()
    }

    fn new_buf(size: usize) -> Vec<u8> {
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
        let result = self.reader.poll_read(self.buf.as_mut_slice())?;

        let mut nb = Self::new_buf(self.size());
        std::mem::swap(&mut nb, &mut self.buf);
        Ok(Async::Ready(Some(nb)))
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
        fn read(&mut self, _buf: &mut [u8]) -> std::io::Result<usize> {
            panic!("should not be called directly");
        }
    }

    impl AsyncRead for FakeRead {
        fn poll_read(&mut self, buf: &mut [u8]) -> Poll<usize, std::io::Error> {
            assert!(!self.input.is_empty());

            match self.input.remove(0) {
                Ok(Async::NotReady) => Ok(Async::NotReady),
                Ok(Async::Ready(b)) => {
                    assert!(buf.len() >= b.len());
                    let len = std::cmp::min(buf.len(), b.len());
                    let b = b.as_slice();
                    buf[..len].clone_from_slice(&b);

                    Ok(Async::Ready(len))
                },
                Err(e) => Err(e),
            }
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
