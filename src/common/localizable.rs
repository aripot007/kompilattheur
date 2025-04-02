/// Represents something that can be localized in the source file
pub trait Localizable {
    /// Get the length of the element
    fn get_len(&self) -> usize;

    /// Get the starting line in the source code
    fn get_start_line(&self) -> usize;

    /// Get the end line in the source code
    fn get_end_line(&self) -> usize;

    /// Get the starting char index on the start line
    fn get_start_char(&self) -> usize;

    /// Get the ending char index on the end line
    fn get_end_char(&self) -> usize;
}

/// Convert any Localizable to a LocalizationInfo.
/// Useful to avoid ownership problems when you need
/// to pass localization information without dropping ownership
macro_rules! localization_info {
    ($l: expr) => {
        LocalizationInfo {
            len: $l.get_len(),
            start_line: $l.get_start_line(),
            end_line: $l.get_end_line(),
            start_char: $l.get_start_char(),
            end_char: $l.get_end_char(),
        }
    };
}
pub(crate) use localization_info;

pub struct LocalizationInfo {
    pub len: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub start_char: usize,
    pub end_char: usize,
}

impl Localizable for LocalizationInfo {
    fn get_len(&self) -> usize {
        self.len
    }

    fn get_start_line(&self) -> usize {
        self.start_line
    }

    fn get_end_line(&self) -> usize {
        self.end_line
    }

    fn get_start_char(&self) -> usize {
        self.start_char
    }

    fn get_end_char(&self) -> usize {
        self.end_char
    }
}
