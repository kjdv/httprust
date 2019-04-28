use std::io::{Read, Write};
use std::io::Result;
use std::str;


pub struct FakeStream {
    pub input: String,
    pub output: String,
    writebuffer: Vec<u8>,
}

impl FakeStream {
    pub fn new(input: &str) -> FakeStream {
        FakeStream {
            input: input.to_string(),
            output: String::new(),
            writebuffer: vec![],
        }
    }
}

impl Read for FakeStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let r: String = self.input.drain(..buf.len()).collect();
        r.as_bytes().read(buf)
    }
}

impl Write for FakeStream {
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
        let mut stream = FakeStream::new("abcdef");

        let mut first = [0; 2];
        assert_eq!(2, stream.read(&mut first).unwrap());
        assert_eq!("ab", str::from_utf8(&first).unwrap());

        let mut second = [0; 4];
        assert_eq!(4, stream.read(&mut second).unwrap());
        assert_eq!("cdef", str::from_utf8(&second).unwrap());
    }

    #[test]
    fn test_fakestream_write() {
        let mut stream = FakeStream::new("");

        assert_eq!(2, stream.write("ab".as_bytes()).unwrap());
        assert_eq!(4, stream.write("cdef".as_bytes()).unwrap());
        assert_eq!("", stream.output.as_str());

        stream.flush().unwrap();
        assert_eq!("abcdef", stream.output.as_str());

        stream.write("ghi".as_bytes()).unwrap();
        stream.flush().unwrap();
        assert_eq!("abcdefghi", stream.output.as_str());
    }
}
