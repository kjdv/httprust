use std::io::{Read, Write};
use std::io::Result;
use httparse;

use super::fakestream::FakeStream;


pub fn handle<T>(mut stream: T) -> Result<()> where T: Read + Write {
    let mut buf = [0; 512];
    let len = stream.read(&mut buf)?;
    stream.write_all(&buf[..len])?;
    stream.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle() {
        let mut stream = FakeStream::new();
        handle(stream.streamer(String::from("blah"))).unwrap();

        assert_eq!("blah", stream.output.as_str());
    }
}
