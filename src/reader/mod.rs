use std::fs::File;
use std::io::prelude::Read;

pub struct Reader {
    file: File,
}

impl Iterator for Reader {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0; 4];
        let mut bytes = self.file.by_ref().bytes();
        let mut char_len = 0;

        while char_len < buffer.len() {
            match bytes.next() {
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

pub fn reader(file_path: String) -> Reader {
    let file = match File::open(&file_path) {
        Ok(file) => file,
        Err(e) => panic!("Error opening file: {e}"),
    };

    Reader { file }
}
