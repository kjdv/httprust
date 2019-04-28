use std::io::{Read, Write};
use std::io::Result;
use std::cmp::max;
use httparse;

pub fn handle<T>(mut stream: T) -> Result<()> where T: Read {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeStream {
        input: Vec<u8>,
    }

    impl FakeStream {
        fn new(input: &str) -> FakeStream {
            FakeStream {
                input: Vec::from(input.as_bytes())
            }
        }
    }

    impl Read for FakeStream {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            let num = max(buf.len(), self.input.len());

            let result: Vec<u8> = self.input.iter().take(num).cloned().collect();
            buf.copy_from_slice(result.as_slice());

            Ok(num)
        }
    }

    #[test]
    fn test_handle() {
        let input = "blah".as_bytes();

        handle(input).unwrap();
    }
}
