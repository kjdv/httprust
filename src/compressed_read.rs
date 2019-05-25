use flate2::read::GzEncoder;
use tokio::io::{AsyncRead};
use std::io::Read;

pub struct CompressedRead<R: AsyncRead>(GzEncoder<R>);

impl<R> CompressedRead<R> where R: AsyncRead {
    pub fn new(base: R) -> CompressedRead<R> {
        CompressedRead(GzEncoder::new(base, flate2::Compression::default()))
    }
}

impl<R> Read for CompressedRead<R> where R: AsyncRead {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}

impl<R> AsyncRead for CompressedRead<R> where R: AsyncRead {}
