use std::fs::File;
use std::io::prelude::Read;

pub struct Reader {
    file: File,
}

impl Iterator for Reader {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = [0; 1];
        match self.file.read(&mut buffer) {
            Ok(0) => None,
            Ok(_) => Some(buffer[0] as char),
            Err(e) => panic!("Error reading file: {e}"),
        }
    }
}

pub fn reader(file_path: String) -> Reader {
    let file = match File::open(&file_path) {
        Ok(file) => file,
        Err(e) => panic!("Error opening file: {e}"),
    };

    Reader { file }
}
