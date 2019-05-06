
use super::log;
use httparse;

use std::io::{Error, ErrorKind, Result};
use std::io::{Read, Write};


pub fn handle<T>(mut stream: T) -> Result<()>
where
    T: Read + Write,
{
    log::info!("handling request");

    const BUFSIZE: usize = 8192;
    let mut read_buffer = [0; BUFSIZE];
    let mut read_pos = 0;

    let _request = loop {
        if read_pos >= BUFSIZE {
            log::error!("request too large");
            return Err(Error::new(
                ErrorKind::Other,
                format!("request too large (max {} bytes)", BUFSIZE),
            ));
        }

        let r = stream.read(&mut read_buffer[read_pos..])?;
        log::debug!("read {} bytes at pos {}", r, read_pos);

        read_pos += r;

        let mut headers = [httparse::EMPTY_HEADER; 16];
        let mut req = httparse::Request::new(&mut headers);

        let result = req.parse(&read_buffer).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("could not parse request: '{}'", e),
            )
        })?;

        if result.is_complete() {
            break result;
        }
    };

    log::debug!("complete request");

    let response = b"HTTP/1.1 200 OK\r\n\r\nhello!\r\n";
    stream.write_all(response)?;
    stream.flush()?;

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::super::fakestream::FakeStream;
    use super::*;

    #[test]
    fn test_handle() {
        let mut stream = FakeStream::new();
        handle(stream.streamer(String::from("GET /index.html HTTP/1.1\r\n\r\n"))).unwrap();

        assert_eq!("HTTP/1.1 200 OK\r\n\r\nhello!\r\n", stream.output.as_str());
    }

    #[test]
    fn test_incomplete_request_errors() {
        let mut stream = FakeStream::new();
        handle(stream.streamer(String::from("GET /index.html"))).unwrap_err();
    }
}
