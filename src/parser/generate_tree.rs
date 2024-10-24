use crate::lexer::lexer::Lexer;
use crate::parser::analysis_table::get_analysis_table;

use super::{analysis_table::AnalysisTable, lexem::Lexem};

pub fn generate_tree(mut lexer: Lexer) -> (bool, bool) {
    let analysis_table: AnalysisTable = get_analysis_table();
    let mut stack: Vec<&Lexem> = vec![&Lexem::NonTerminal(0)];
    let mut error = false;
    let mut accept = false;
    let mut input = lexer.next().unwrap();

    while !error && !accept {
        println!("Stack: {stack:?}, Input: {input}");
        let x = stack.pop();
        match x {
            Some(Lexem::Terminal(token)) => {
                if *token != input {
                    error = true;
                } else {
                    input = lexer.next().unwrap();
                }
            }
            Some(Lexem::NonTerminal(id)) => {
                let entry = analysis_table.get(id, &input);
                match entry {
                    Some(lexems) => {
                        for lexem in lexems.iter().rev() {
                            stack.push(lexem);
                        }
                    }
                    None => {
                        error = true;
                    }
                }
            }
            None => {
                accept = true;
            }
        }
    }
    return (accept, error);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_tree() {
        let lexer = Lexer::new("1 + 1 * 1".into());
        for token in lexer {
            print!("{}", token);
        }
        print!("\n");
        let lexer = Lexer::new("1 + 1 * 1".into());
        let (accept, error) = generate_tree(lexer);
        assert_eq!(accept, true);
        assert_eq!(error, false);
    }
}