use std::error::Error;
use std::io::Cursor;
use std::io::Result;
use std::io::{Read, Write};


type ByteStream = Cursor<Vec<u8>>;

pub enum Item {
    Data(Vec<u8>),
    Error(std::io::Error),
}

// shorthand
pub fn make_data(buf: &'static [u8]) -> Item {
    Item::Data(Vec::from(buf))
}

pub fn make_error(message: &str) -> Item {
    Item::Error(std::io::Error::new(std::io::ErrorKind::Other, message))
}

pub struct FakeStream {
    pub output: Vec<u8>,
}

impl FakeStream {
    pub fn new() -> FakeStream {
        FakeStream { output: Vec::new() }
    }

    pub fn streamer(&mut self, input: Vec<Item>) -> Streamer {
        Streamer {
            input: input
                .into_iter()
                .map(|i| match i {
                    Item::Data(d) => StreamItem::Data(ByteStream::new(d)),
                    Item::Error(e) => StreamItem::Error(e),
                })
                .collect(),

            output: &mut self.output,
            writebuffer: ByteStream::new(vec![]),
        }
    }
}

enum StreamItem {
    Data(ByteStream),
    Error(std::io::Error),
}

pub struct Streamer<'a> {
    input: Vec<StreamItem>,
    output: &'a mut Vec<u8>,
    writebuffer: ByteStream,
}

impl<'a> Read for Streamer<'a> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.input.is_empty() {
            return Ok(0);
        }

        let mut head = self.input.remove(0);
        let (result, should_drop) = read_head(&mut head, buf);

        if !should_drop {
            self.input.insert(0, head)
        }

        result
    }
}

fn read_head(head: &mut StreamItem, buf: &mut [u8]) -> (Result<usize>, bool) {
    match head {
        StreamItem::Data(ref mut d) => match d.read(buf) {
            Ok(s) => (Ok(s), d.position() == d.get_ref().len() as u64),
            Err(e) => (Err(e), true),
        },
        StreamItem::Error(e) => (Err(std::io::Error::new(e.kind(), e.description())), true),
    }
}

impl<'a> Write for Streamer<'a> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.writebuffer.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.writebuffer.set_position(0);
        let mut tmp = String::new();
        self.writebuffer.read_to_string(&mut tmp).unwrap();
        self.writebuffer = ByteStream::new(vec![]);

        self.output.extend(tmp.into_bytes());

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
        let mut streamer = stream.streamer(vec![make_data(b"abcdef")]);

        let mut first = [0; 2];
        assert_eq!(2, streamer.read(&mut first).unwrap());
        assert_eq!("ab", str::from_utf8(&first).unwrap());

        let mut second = [0; 4];
        assert_eq!(4, streamer.read(&mut second).unwrap());
        assert_eq!("cdef", str::from_utf8(&second).unwrap());
    }

    #[test]
    fn test_read_multiple() {
        let mut stream = FakeStream::new();
        let mut streamer = stream.streamer(vec![make_data(b"abc"), make_data(b"def")]);

        let mut buf = [0; 6];
        assert_eq!(3, streamer.read(&mut buf).unwrap());
        assert_eq!(3, streamer.read(&mut buf[3..]).unwrap());
        assert_eq!(b"abcdef", &buf);
    }

    #[test]
    fn test_read_until_error() {
        let mut stream = FakeStream::new();
        let mut streamer = stream.streamer(vec![
            make_data(b"abc"),
            make_error("booh"),
            make_data(b"def"),
        ]);

        let mut buf = [0; 6];

        streamer.read(&mut buf).unwrap();
        streamer.read(&mut buf[3..]).unwrap_err();
        streamer.read(&mut buf[3..]).unwrap();

        assert_eq!(b"abcdef", &buf);
    }

    #[test]
    fn test_fakestream_write() {
        let mut stream = FakeStream::new();
        let mut streamer = stream.streamer(vec![]);

        assert_eq!(2, streamer.write(b"ab").unwrap());
        assert_eq!(4, streamer.write(b"cdef").unwrap());
        assert_eq!(b"", streamer.output.as_slice());

        streamer.flush().unwrap();
        assert_eq!(b"abcdef", streamer.output.as_slice());

        streamer.write(b"ghi").unwrap();
        streamer.flush().unwrap();
        assert_eq!(b"abcdefghi", streamer.output.as_slice());
    }
}
