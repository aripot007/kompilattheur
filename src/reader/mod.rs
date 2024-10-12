pub struct FileChars {
    index: usize,
    size: usize,
    file_path: String,
}

impl Iterator for FileChars {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.size {
            self.index += 1;
            let res = std::fs::read_to_string(&self.file_path)
                .ok()
                .and_then(|s| s.chars().nth(self.index - 1));
            res
        } else {
            None
        }
    }
}

pub fn file_chars(file_path: String) -> FileChars {
    if !std::path::Path::new(&file_path).exists() {
        panic!("File does not exist: {}", file_path);
    }

    let size = std::fs::read_to_string(&file_path)
        .map(|s| s.chars().count())
        .unwrap_or(0);

    FileChars {
        index: 0,
        size,
        file_path,
    }
}
