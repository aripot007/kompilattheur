use std::collections::VecDeque;
use std::fs::File;
use std::io;
use std::io::prelude::Read;
use std::path::Path;

pub struct Reader {
    bytes: Box<dyn Iterator<Item = Result<u8, io::Error>>>,
}

impl Iterator for Reader {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0; 4];
        let mut char_len = 0;

        while char_len < buffer.len() {
            match self.bytes.next() {
                Some(Ok(byte)) => {
                    buffer[char_len] = byte;
                    char_len += 1;
                    // get the string from the buffer
                    if let Ok(s) = std::str::from_utf8(&buffer[..char_len]) {
                        // get the first char
                        if let Some(c) = s.chars().next() {
                            return Some(c);
                        }
                    }
                }
                Some(Err(e)) => panic!("Error reading file: {e}"),
                None => return None,
            }
        }
        None
    }
}

pub fn new(file_path: &Path) -> Reader {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => panic!("Error opening file: {e}"),
    };

    let bytes = Box::new(file.bytes());

    Reader { bytes }
}

struct StringReader {
    b: VecDeque<u8>,
}

impl StringReader {
    fn new(s: &str) -> Self {
        let b = VecDeque::from(Vec::from(s.as_bytes()));
        return StringReader { b };
    }
}

impl Iterator for StringReader {
    type Item = Result<u8, io::Error>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.b.pop_front() {
            Some(c) => Some(Ok(c)),
            None => None,
        }
    }
}

impl From<&str> for Reader {
    fn from(value: &str) -> Self {
        let bytes = Box::new(StringReader::new(value));
        return Reader { bytes };
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;

    macro_rules! test_read {
        ($s: expr) => {
            let m = $s;
            let r = Reader::from(m);
            let s: String = r.collect();
            assert!(m == s);
        };
    }

    #[test]
    fn test_ascii() {
        test_read!("ascii file");
    }

    #[test]
    fn test_emoji() {
        test_read!("🐍");
    }

    #[test]
    fn test_long_utf8_char() {
        test_read!("こんにちは、世界!");
    }
}
