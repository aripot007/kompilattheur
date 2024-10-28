use colored::Colorize;
use std::collections::LinkedList;
use std::fs::File;
use std::io::{self, BufRead};

use crate::FILE_PATH;

const error_color:(u8,u8,u8) = (255, 0, 0);
const warning_color:(u8,u8,u8) = (255, 130, 0);
const type_color:(u8,u8,u8) = (0, 165, 255);
const highlight_color:(u8,u8,u8) = (150, 0, 0);
const secondary_color:(u8,u8,u8) = (125, 125, 125);

pub enum DiagnosticGravity {
    Warning,
    Error,
}

pub struct Diagnostic {
    gravity: DiagnosticGravity,
    kind: String,
    start_line: u64,
    end_line: u64,
    start_column: u64,
    end_column: u64,
    message: String,
}

impl Diagnostic {
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

    pub fn display(&self) {
        match self.gravity {
            DiagnosticGravity::Warning => {
                println!(
                    "{} {} \n{}\n{} {}",
                    "Warning :".truecolor(warning_color.0,warning_color.1,warning_color.2).bold(),
                    format!("at line {}:{} :", self.start_line, self.start_column).bold(),
                    self.format_source_line(),
                    self.kind.truecolor(type_color.0,type_color.1,type_color.2),
                    self.message
                );
            }
            DiagnosticGravity::Error => {
                println!(
                    "{} {} \n{}\n{} {}",
                    "Error :".truecolor(error_color.0,error_color.1,error_color.2).bold(),
                    format!("at line {}:{} :", self.start_line, self.start_column).bold(),
                    self.format_source_line(),
                    self.kind.truecolor(type_color.0,type_color.1,type_color.2),
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
                            result.push_str(format!("{} {} {}", (i+1).to_string().truecolor(secondary_color.0, secondary_color.1, secondary_color.2),"|".truecolor(secondary_color.0, secondary_color.1, secondary_color.2) , line.unwrap()).to_string().as_str());
                            result.push('\n');
                            let spaces = self.start_column as usize+ i.to_string().len() + 3;
                            result.push_str(format!("{:<spaces$}{:^>token_len$}", "", "", token_len = self.end_column as usize - self.start_column as usize).truecolor(highlight_color.0, highlight_color.1, highlight_color.2).to_string().as_str());
                        }
                    }
                }
               else {
                   for (i,line) in lines.enumerate() {
                       let line = line.unwrap();
                       if i+1 == self.start_line as usize {
                            result.push_str(format!("{} {} {}", (i+1).to_string().truecolor(secondary_color.0, secondary_color.1, secondary_color.2),"|".truecolor(secondary_color.0, secondary_color.1, secondary_color.2) , line).to_string().as_str());
                            result.push('\n');
                            let spaces = self.start_column as usize+ i.to_string().len() + 3;
                            result.push_str(format!("{:<spaces$}{:^>token_len$}", "", "", token_len = line.len() - self.start_column as usize).truecolor(highlight_color.0, highlight_color.1, highlight_color.2).to_string().as_str());
                            result.push('\n');
                        } else if i+1 > self.start_line as usize && i+1 < self.end_line as usize {
                            result.push_str(format!("{} {} {}", (i+1).to_string().truecolor(secondary_color.0, secondary_color.1, secondary_color.2),"|".truecolor(secondary_color.0, secondary_color.1, secondary_color.2) , line).to_string().as_str());
                            result.push('\n');
                            let spaces = i.to_string().len() + 3;
                            result.push_str(format!("{:<spaces$}{:^>token_len$}", "", "", token_len = line.len()).truecolor(highlight_color.0, highlight_color.1, highlight_color.2).to_string().as_str());
                            result.push('\n');
                        } else if i+1 == self.end_line as usize {
                            result.push_str(format!("{} {} {}", (i+1).to_string().truecolor(secondary_color.0, secondary_color.1, secondary_color.2),"|".truecolor(secondary_color.0, secondary_color.1, secondary_color.2) , line).to_string().as_str());
                            result.push('\n');
                            let spaces = i.to_string().len() + 3;
                            result.push_str(format!("{:<spaces$}{:^>token_len$}", "", "", token_len = self.end_column as usize).truecolor(highlight_color.0, highlight_color.1, highlight_color.2).to_string().as_str());
                        }
                   }
               } 
            }
        }
        result
    }
}
