use analysis_table::AnalysisTable;
use crate::lexer::lexer::Lexer;

pub mod analysis_table;
pub mod analysis_table_generator;
pub mod lexem;

pub fn generate_tree(lexer: Lexer, analysis_table: AnalysisTable) {}
