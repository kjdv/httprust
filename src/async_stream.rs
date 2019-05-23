use futures::{Stream, Poll};
use futures::prelude::*;
use tokio::io::{AsyncRead};


pub struct AsyncStream<A>
    where A: AsyncRead {
    io: A,
    buffer: Vec<u8>,
    pos: usize,
}

impl<A> AsyncStream<A>
    where A: AsyncRead {

    pub fn new(reader: A) -> AsyncStream<A> {
        const DEFAULT_SIZE: usize = 0xffff;
        AsyncStream::with_size(reader, DEFAULT_SIZE)
    }

    pub fn with_size(reader: A, max_size: usize) -> AsyncStream<A> {
        AsyncStream{
            io: reader,
            buffer: Self::make_buf(max_size),
            pos: 0,
        }
    }

    fn size(&self) -> usize {
        self.buffer.len()
    }

    fn make_buf(size: usize) -> Vec<u8> {
        let mut nb = Vec::with_capacity(size);
        nb.resize(size, 0);
        nb
    }

    fn reset_buffer(&mut self) -> Vec<u8> {
        let new_buf = Self::make_buf(self.size());
        let mut result = std::mem::replace(&mut self.buffer, new_buf);
        result.truncate(self.pos);
        self.pos = 0;
        result
    }
}

impl<A> Stream for AsyncStream<A>
    where A: AsyncRead {
    type Item = Vec<u8>;
    type Error = std::io::Error;

    fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
        let start = self.pos;
        let end = self.size();

        assert!(end > start);

        let request = end - start;

        match self.io.read(&mut self.buffer.as_mut_slice()[start..end]) {
            Ok(n) if n == request => { // all available
                self.pos += n;
                let result = self.reset_buffer();
                Ok(Async::Ready(Some(result)))
            },
            Ok(n) if n > 0 => { // part available
                assert!(n < request);
                self.pos += n;
                Ok(Async::NotReady)
            },
            Ok(0) => { // eof
                // if there is something in the buffer, return it
                if self.pos > 0 {
                    let result = self.reset_buffer();
                    Ok(Async::Ready(Some(result)))
                } else {
                    Ok(Async::Ready(None))
                }
            },
            Ok(_) => panic!("unreachable"),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                Ok(Async::NotReady)
            },
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;

    enum Part {
        Data(Vec<u8>),
        Block,
        Eof,
        Err(std::io::Error),
    }

    impl Part {
        fn data(b: &'static [u8]) -> Part {
            Part::Data(Vec::from(b))
        }

        fn err(msg: &str) -> Part {
            Part::Err(std::io::Error::new(std::io::ErrorKind::Other, msg))
        }
    }

    struct FakeRead {
        input: Vec<Part>
    }

    impl FakeRead {
        fn new(input: Vec<Part>) -> FakeRead {
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
                Part::Data(data) => {
                    let len = std::cmp::min(buf.len(), data.len());
                    let data = data.as_slice();
                    buf[..len].clone_from_slice(&data[..len]);

                    Ok(len)
                },
                Part::Block => Err(wouldblock()),
                Part::Eof => Ok(0),
                Part::Err(e) => Err(e),
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
                Part::data(b"abcd")
            ]
        ), 4);

        let result = stream.poll().unwrap();
        assert_eq!(make_ready("abcd"), result);
    }

    #[test]
    fn underflow() {
        let mut stream = AsyncStream::with_size(
            FakeRead::new(vec![
                Part::data(b"abc"),
                Part::data(b"d"),
            ]
        ), 4);

        let result = stream.poll().unwrap();
        assert!(!result.is_ready());

        let result = stream.poll().unwrap();
        assert_eq!(make_ready("abcd"), result);
    }

    #[test]
    fn overflow() {
        let mut stream = AsyncStream::with_size(
            FakeRead::new(vec![
                Part::data(b"abcdef"),
            ]
        ), 4);

        let result = stream.poll().unwrap();
        assert_eq!(make_ready("abcd"), result);
    }

    #[test]
    fn wouldblock_is_noop() {
        let mut stream = AsyncStream::with_size(
            FakeRead::new(vec![
                Part::Block,
            ]
        ), 4);

        let result = stream.poll().unwrap();
        assert!(!result.is_ready());
    }

    #[test]
    fn error_is_propagated() {
        let mut stream = AsyncStream::with_size(
            FakeRead::new(vec![
                Part::err("blah"),
            ]
        ), 4);

        stream.poll().unwrap_err();
    }

    #[test]
    fn eof_gives_none() {
        let mut stream = AsyncStream::with_size(
            FakeRead::new(vec![
                Part::Eof,
            ]
        ), 4);

        let result = stream.poll().unwrap();
        assert_eq!(Async::Ready(None), result);
    }

    #[test]
    fn early_eof_gives_part_chunk() {
        let mut stream = AsyncStream::with_size(
            FakeRead::new(vec![
                Part::data(b"ab"),
                Part::Eof,
            ]
        ), 4);

        let result = stream.poll().unwrap();
        assert!(!result.is_ready());

        let result = stream.poll().unwrap();
        assert_eq!(make_ready("ab"), result);

        let result = stream.poll().unwrap();
        assert_eq!(Async::Ready(None), result);
    }

    #[test]
    fn buffer_is_reset() {
        let mut stream = AsyncStream::with_size(
            FakeRead::new(vec![
                Part::data(b"abcd"),
                Part::data(b"ef"),
                Part::data(b"gh"),
                Part::Eof,
            ]
        ), 4);

        let result = stream.poll().unwrap();
        assert_eq!(make_ready("abcd"), result);

        let result = stream.poll().unwrap();
        assert!(!result.is_ready());

        let result = stream.poll().unwrap();
        assert_eq!(make_ready("efgh"), result);

        let result = stream.poll().unwrap();
        assert_eq!(Async::Ready(None), result);
    }
}
