
use std::io::Cursor;
use std::io::Result;
use std::io::{Read, Write};

type ByteStream = Cursor<Vec<u8>>;

pub struct FakeStream {
    pub output: String,
}

impl FakeStream {
    pub fn new() -> FakeStream {
        FakeStream {
            output: String::new(),
        }
    }

    pub fn streamer(&mut self, input: String) -> Streamer {
        Streamer {
            input: ByteStream::new(Vec::from(input)),
            output: &mut self.output,
            writebuffer: ByteStream::new(vec![]),
        }
    }
}

pub struct Streamer<'a> {
    input: ByteStream,
    output: &'a mut String,
    writebuffer: ByteStream,
}

impl<'a> Read for Streamer<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.input.read(buf)
    }
}

impl<'a> Write for Streamer<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.writebuffer.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.writebuffer.set_position(0);
        self.writebuffer.read_to_string(self.output).unwrap();
        self.writebuffer = ByteStream::new(vec![]);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn test_fakestream_read() {
        let mut stream = FakeStream::new();
        let mut streamer = stream.streamer(String::from("abcdef"));

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
        let mut streamer = stream.streamer(String::new());

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
