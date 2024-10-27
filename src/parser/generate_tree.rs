use crate::lexer::lexer::Lexer;
use super::lexem::Lexem;
use crate::analysis_table::analysis_table::AnalysisTable;

pub fn generate_tree(mut lexer: Lexer, analysis_table: &AnalysisTable) -> (bool, bool) {
    let mut stack: Vec<&Lexem> = vec![&Lexem::NonTerminal(0)];
    let mut error = false;
    let mut accept = false;
    let mut input = lexer.next().unwrap();

    while !error && !accept {
        println!("Stack: {stack:?}, Input: {input}");
        let x = stack.pop();
        match x {
            Some(Lexem::Terminal(token)) => {
                if token.is_same_type(&input) {
                    input = lexer.next().unwrap();
                } else {
                    error = true;
                    println!("Error: {token:?} != {input}");
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
                        println!("Error: No entry for {id:?} and {input}");
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


    use crate::analysis_table::analysis_table_generator::generate_analysis_table;
    use std::path::PathBuf;

    use super::*;
    
    #[test]
    fn test_generate_tree() {
        let source = "1 + (1 * 1)";
        let lexer = Lexer::new(source.into());
        for token in lexer {
            print!("{}", token);
        }
        print!("\n");
        let lexer = Lexer::new(source.into());
        let (accept, error) = generate_tree(lexer, &generate_analysis_table(&PathBuf::from("grammaire_ex.txt")));
        assert_eq!(accept, true);
        assert_eq!(error, false);
    }
}