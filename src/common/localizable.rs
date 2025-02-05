/// Represents something that can be localized in the source file
pub trait Localizable {
    /// Get the starting line in the source code
    fn get_start_line(&self) -> usize;

    /// Get the end line in the source code
    fn get_end_line(&self) -> usize;

    /// Get the starting char index on the start line
    fn get_start_char(&self) -> usize;

    /// Get the ending char index on the end line
    fn get_end_char(&self) -> usize;
}
