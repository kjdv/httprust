use std::io::{Read, Write};
use std::io::Result;
use std::str;


pub struct FakeStream {
    pub output: String,
}

impl FakeStream {
    pub fn new() -> FakeStream {
        FakeStream {
            output: String::new(),
        }
    }

    pub fn streamer(&mut self, input: &str) -> Streamer {
        Streamer {
            input: input.to_string(),
            output: &mut self.output,
            writebuffer: vec![],
        }
    }
}

pub struct Streamer<'a> {
    input: String,
    output: &'a mut String,
    writebuffer: Vec<u8>,
}

impl<'a> Read for Streamer<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let num = std::cmp::min(buf.len(), self.input.len());
        let r: String = self.input.drain(..num).collect();
        r.as_bytes().read(buf)
    }
}

impl<'a> Write for Streamer<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.writebuffer.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        let s = std::str::from_utf8(self.writebuffer.as_slice()).unwrap();
        self.output.extend(s.chars());
        self.writebuffer.clear();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fakestream_read() {
        let mut stream = FakeStream::new();
        let mut streamer = stream.streamer("abcdef");

        let mut first = [0; 2];
        assert_eq!(2, streamer.read(&mut first).unwrap());
        assert_eq!("ab", str::from_utf8(&first).unwrap());

        let mut second = [0; 4];
        assert_eq!(4, streamer.read(&mut second).unwrap());
        assert_eq!("cdef", str::from_utf8(&second).unwrap());
    }

    #[test]
    fn test_fakestream_write() {
        let mut stream = FakeStream::new();
        let mut streamer = stream.streamer("");

        assert_eq!(2, streamer.write("ab".as_bytes()).unwrap());
        assert_eq!(4, streamer.write("cdef".as_bytes()).unwrap());
        assert_eq!("", streamer.output.as_str());

        streamer.flush().unwrap();
        assert_eq!("abcdef", streamer.output.as_str());

        streamer.write("ghi".as_bytes()).unwrap();
        streamer.flush().unwrap();
        assert_eq!("abcdefghi", streamer.output.as_str());
    }
}
