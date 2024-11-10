use colored::Colorize;
use std::fs::File;
use std::io::{self, BufRead};

use crate::FILE_PATH;

const ERROR_COLOR:(u8,u8,u8) = (255, 0, 0);
const WARNING_COLOR:(u8,u8,u8) = (255, 130, 0);
const TYPE_COLOR:(u8,u8,u8) = (0, 165, 255);
const HIGHLIGHT_COLOR:(u8,u8,u8) = (150, 0, 0);
const SECONDARY_COLOR:(u8,u8,u8) = (125, 125, 125);

pub enum DiagnosticGravity {
    Warning,
    Error,
}

pub struct Diagnostic {
    pub gravity: DiagnosticGravity,
    kind: String,
    start_line: u64,
    end_line: u64,
    start_column: u64,
    end_column: u64,
    message: String,
}

impl Diagnostic {
    /// Create a new Diagnostic
    /// 
    /// # Arguments
    /// 
    /// * `gravity` - between Warning and Error
    /// * `kind` - describes the type of error, for example: "IntOverflow"
    /// * `start_line` - line where the error starts
    /// * `end_line` - line where the error ends
    /// * `start_column` - character of start_line where the error starts
    /// * `end_column` - character of end_line where the error ends
    /// * `message` - message to display
    pub fn new(
        gravity: DiagnosticGravity,
        kind: String,
        start_line: u64,
        end_line: u64,
        start_column: u64,
        end_column: u64,
        message: String,
    ) -> Self {
        Self {
            gravity,
            kind,
            start_line,
            end_line,
            start_column,
            end_column,
            message,
        }
    }

    /// Display the diagnostic in the terminal
    pub fn display(&self) {
        match self.gravity {
            DiagnosticGravity::Warning => {
                println!(
                    "{} {} \n{}\n{} {}",
                    "Warning :".truecolor(WARNING_COLOR.0,WARNING_COLOR.1,WARNING_COLOR.2).bold(),
                    format!("at line {}:{} :", self.start_line, self.start_column).bold(),
                    self.format_source_line(),
                    self.kind.truecolor(TYPE_COLOR.0,TYPE_COLOR.1,TYPE_COLOR.2),
                    self.message
                );
            }
            DiagnosticGravity::Error => {
                println!(
                    "{} {} \n{}\n{} {}",
                    "Error :".truecolor(ERROR_COLOR.0,ERROR_COLOR.1,ERROR_COLOR.2).bold(),
                    format!("at line {}:{} :", self.start_line, self.start_column).bold(),
                    self.format_source_line(),
                    self.kind.truecolor(TYPE_COLOR.0,TYPE_COLOR.1,TYPE_COLOR.2),
                    self.message
                );
            }
        }
    }
    
    fn format_source_line(&self) -> String {
        let mut result = String::new();
        if let Some(path) = FILE_PATH.get() {
            if let Ok(file) = File::open(path) {
                let reader = io::BufReader::new(file);
                let lines = reader.lines();
                if self.start_line == self.end_line {
                    for (i, line) in lines.enumerate() {
                        if i+1 == self.start_line as usize {
                            result.push_str(format!("{} {} {}", (i+1).to_string().truecolor(SECONDARY_COLOR.0, SECONDARY_COLOR.1, SECONDARY_COLOR.2),"|".truecolor(SECONDARY_COLOR.0, SECONDARY_COLOR.1, SECONDARY_COLOR.2) , line.unwrap()).to_string().as_str());
                            result.push('\n');
                            let spaces = self.start_column as usize+ i.to_string().len() + 2;
                            result.push_str(format!("{:<spaces$}{:^>token_len$}", "", "", token_len = self.end_column as usize - self.start_column as usize + 1).truecolor(HIGHLIGHT_COLOR.0, HIGHLIGHT_COLOR.1, HIGHLIGHT_COLOR.2).to_string().as_str());
                        }
                    }
                }
               else {
                   for (i,line) in lines.enumerate() {
                       let line = line.unwrap();
                       if i+1 == self.start_line as usize {
                            result.push_str(format!("{} {} {}", (i+1).to_string().truecolor(SECONDARY_COLOR.0, SECONDARY_COLOR.1, SECONDARY_COLOR.2),"|".truecolor(SECONDARY_COLOR.0, SECONDARY_COLOR.1, SECONDARY_COLOR.2) , line).to_string().as_str());
                            result.push('\n');
                            let spaces = self.start_column as usize+ i.to_string().len() + 2;
                            result.push_str(format!("{:<spaces$}{:^>token_len$}", "", "", token_len = line.len() - self.start_column as usize).truecolor(HIGHLIGHT_COLOR.0, HIGHLIGHT_COLOR.1, HIGHLIGHT_COLOR.2).to_string().as_str());
                            result.push('\n');
                        } else if i+1 > self.start_line as usize && i+1 < self.end_line as usize {
                            result.push_str(format!("{} {} {}", (i+1).to_string().truecolor(SECONDARY_COLOR.0, SECONDARY_COLOR.1, SECONDARY_COLOR.2),"|".truecolor(SECONDARY_COLOR.0, SECONDARY_COLOR.1, SECONDARY_COLOR.2) , line).to_string().as_str());
                            result.push('\n');
                            let spaces = i.to_string().len() + 3;
                            result.push_str(format!("{:<spaces$}{:^>token_len$}", "", "", token_len = line.len()).truecolor(HIGHLIGHT_COLOR.0, HIGHLIGHT_COLOR.1, HIGHLIGHT_COLOR.2).to_string().as_str());
                            result.push('\n');
                        } else if i+1 == self.end_line as usize {
                            result.push_str(format!("{} {} {}", (i+1).to_string().truecolor(SECONDARY_COLOR.0, SECONDARY_COLOR.1, SECONDARY_COLOR.2),"|".truecolor(SECONDARY_COLOR.0, SECONDARY_COLOR.1, SECONDARY_COLOR.2) , line).to_string().as_str());
                            result.push('\n');
                            let spaces = i.to_string().len() + 3;
                            result.push_str(format!("{:<spaces$}{:^>token_len$}", "", "", token_len = self.end_column as usize + 1).truecolor(HIGHLIGHT_COLOR.0, HIGHLIGHT_COLOR.1, HIGHLIGHT_COLOR.2).to_string().as_str());
                        }
                   }
               } 
            }
        }
        result
    }
}
