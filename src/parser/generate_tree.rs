use crate::lexer::lexer::Lexer;
use crate::parser::analysis_table::get_analysis_table;

use super::{analysis_table::AnalysisTable, lexem::{Lexem, NonTerminal}};

pub fn generate_tree(lexer: Lexer) {
    let analysis_table: AnalysisTable = get_analysis_table();
    let mut stack: Vec<Lexem> = Vec::new();
}