use std::cell::RefCell;
use std::rc::Rc;

use crate::common::types::token::Token;
use crate::{common::types::tree::Node, lexer::lexer::Lexer};
use crate::parser::analysis_table::get_analysis_table;

use super::{analysis_table::AnalysisTable, lexem::Lexem};

pub fn generate_tree(mut lexer: Lexer) -> (bool, bool) {
    let analysis_table: AnalysisTable = get_analysis_table();
    let mut stack: Vec<Rc<RefCell<Node<Lexem>>>> = vec![Node::new(Lexem::NonTerminal(0))];
    let mut error = false;
    let mut accept = false;
    let mut input = lexer.next().unwrap();

    while !error && !accept {
        println!("Stack: {:?}, Input: {}", stack, input);
        let x = stack.pop();
        match x {
            Some(node) => {
                let lexem  = node.borrow_mut().value.clone();
                match lexem {
                    Lexem::Terminal(token) => {
                        if token.is_same_type(&input) {
                            input = lexer.next().unwrap();
                        } else {
                            error = true;
                            println!("Error: {token:?} != {input}");
                        }
                    }
                    Lexem::NonTerminal(id) => {
                        let entry = analysis_table.get(&id, &input);
                        match entry {
                            Some(lexems) => {
                                for lexem in lexems.iter().rev() {
                                    stack.push(Node::new((*lexem).clone()));
                                }
                            }
                            None => {
                                error = true;
                                println!("Error: No entry for {id:?} and {input}");
                            }
                        }
                    }
                }
            }
            None => {
                if input != Token::EOF {
                    error = true;
                    println!("Error: Stack is empty and input is not EOF");
                }
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
        let source = "1 + 1 * 1)";
        let lexer = Lexer::new(source.into());
        for token in lexer {
            print!("{}", token);
        }
        print!("\n");
        let lexer = Lexer::new(source.into());
        let (accept, error) = generate_tree(lexer);
        assert_eq!(accept, true);
        assert_eq!(error, false);
    }
}