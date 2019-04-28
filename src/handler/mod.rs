use std::io::{Read, Write};
use std::io::Result;
use httparse;


pub fn handle<T>(mut stream: &T) -> Result<()> where T: Read + Write {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod fakestream;
    use fakestream::FakeStream;

    #[test]
    fn test_handle() {
        let mut stream = FakeStream::new("blah");
        handle(&stream).unwrap();

        assert_eq!("", stream.output.as_str());
    }
}
