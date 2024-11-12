use colored:: Colorize;

use crate::{common::{diagnostic::{Diagnostic, DiagnosticGravity}, types::file_element}, lexer::Lexer};
use super::lexem::Lexem;
use crate::analysis_table::AnalysisTable;
use std::cell::RefCell;
use std::rc::Rc;
use crate::common::types::{Token, Node};

pub fn generate_tree(mut lexer: Lexer, analysis_table: &AnalysisTable) -> (Rc<RefCell<Node<Lexem>>>, bool, bool) {
    let tree: Rc<RefCell<Node<Lexem>>> = Node::new(Lexem::NonTerminal(0));
    let mut stack: Vec<Rc<RefCell<Node<Lexem>>>> = vec![tree.clone()];
    let mut error = false;
    let mut accept = false;
    let mut input = lexer.next().unwrap_or(file_element::EOF);

    while !error && !accept {
        //println!("Stack: {:?}, Input: {}", stack, input);
        let x = stack.pop();
        //println!("Tree: {:?}", tree.borrow());
        //println!("Node: {:?}", x);
        match x {
            Some(node) => {
                let lexem = node.borrow_mut().value.clone();
                match lexem {
                    Lexem::Terminal(token) => {
                        if token.is_same_type(&input.element) {
                            //println!("Input: {:?}", input);
                            node.borrow_mut().value = Lexem::Terminal(input.element.clone());
                            input = lexer.next().unwrap_or(file_element::EOF);
                        } else {
                            error = true;
                            let line = if input.element == Token::Newline && input.line > 0 {
                                input.line - 1
                            } else {
                                input.line
                            };
                            Diagnostic::new(
                                DiagnosticGravity::Error,
                                "ParserInputError :".to_string(),
                                line,
                                input.line,
                                input.start_char,
                                if input.len > 0 { input.start_char + (input.len - 1) as u64 } else { input.start_char },
                                format!("Expected {} but got {}", token.to_string().truecolor(255, 0, 0), input.element.to_string().truecolor(255, 0, 0)).to_string(),
                            )
                            .display();
                            // println!("Error: {token:?} != {}", input.element);
                        }
                    }
                    Lexem::NonTerminal(id) => {
                        let entry = analysis_table.get(&id, &input.element);
                        match entry {
                            Some(lexems) => {
                                for lexem in lexems.iter().rev() {
                                    let new_node = Node::new((*lexem).clone());
                                    stack.push(new_node.clone());
                                    node.borrow_mut().insert_child(0, new_node.clone());
                                }
                            }
                            None => {
                                error = true;
                                let expected_tokens = analysis_table.get_expected_tokens(&id);
                                let expected_tokens = expected_tokens.iter().map(|x| {
                                    match x {
                                        Token::Identifier(_) => "Identifier".to_string().truecolor(255, 0, 0).to_string(),
                                        Token::Integer(_) => "Integer".to_string().truecolor(255, 0, 0).to_string(),
                                        Token::String(_) => "String".to_string().truecolor(255, 0, 0).to_string(),
                                        _ => x.to_string().truecolor(255, 0, 0).to_string(),
                                    }
                                }).collect::<Vec<String>>().join(", ");
                                Diagnostic::new(
                                    DiagnosticGravity::Error,
                                    "ParserTableError :".to_string(),
                                    input.line,
                                    input.line,
                                    input.start_char,
                                    if input.len > 0 { input.start_char + (input.len - 1) as u64 } else { input.start_char },
                                    format!("Expected {} but got {}", expected_tokens, input.element.to_string().truecolor(255, 0, 0)).to_string(),
                                ).display();
                                //println!("Error: No entry for {} and {}", analysis_table.get_non_terminal_name(id), input.element);
                            }
                        }
                    }
                }
            }
            None => {
                if input.element != Token::EOF {
                    error = true;
                    Diagnostic::new(
                        DiagnosticGravity::Error,
                        "ParserStackError :".to_string(),
                        input.line,
                        input.line,
                        input.start_char,
                        input.start_char + (input.len - 1) as u64,
                        "Stack is empty and input is not EOF".to_string(),
                    ).display();
                    println!("Error: Stack is empty and input is not EOF");
                }
                accept = true;
            }
        }
    }

    // Finish lexical analysis before returning
    if error {
        while let Some(_) = lexer.next() {};
    }

    return (tree, accept, error);
}

#[cfg(test)]
mod tests {


    use crate::analysis_table::setup_analysis_table;
    use std::path::PathBuf;

    use super::*;

    use once_cell::sync::Lazy;

    static ANALYSIS_TABLE: Lazy<&AnalysisTable> = Lazy::new(|| setup_analysis_table(Some(&PathBuf::from("grammaire.txt"))));

    #[test]
    fn test_generate_tree() {
        let source = "5 + 5 * 8 \n";
        // let lexer = Lexer::new(source.into());
        // for token in lexer {
        //     print!("{}", token);
        // }
        // print!("\n");
        let lexer = Lexer::new(source.into());
        let analysis_table = Lazy::force(&ANALYSIS_TABLE);
        let (tree, accept, error) = generate_tree(lexer, *analysis_table);
        println!("{}", tree.borrow().generate_mermaid());
        assert_eq!(accept, true);
        assert_eq!(error, false);
    }

    #[test]
    fn test_invalid_input_parser() {
        let source = "print \"Hello World !\")";
        let lexer = Lexer::new(source.into());
        let analysis_table = Lazy::force(&ANALYSIS_TABLE);
        let (tree, accept, error) = generate_tree(lexer, *analysis_table);
        println!("{}", tree.borrow().generate_mermaid());
        assert_eq!(accept, false);
        assert_eq!(error, true);
    }

    #[test]
    fn test_invalid_rule_parser() {
        let source = "( + ";
        let lexer = Lexer::new(source.into());
        let analysis_table = Lazy::force(&ANALYSIS_TABLE);
        let (tree, accept, error) = generate_tree(lexer, *analysis_table);
        println!("{}", tree.borrow().generate_mermaid());
        assert_eq!(accept, false);
        assert_eq!(error, true);
    }

    #[test]
    fn test_empty_stack_parser() {
        let source = "\n \n";
        let lexer = Lexer::new(source.into());
        let analysis_table = Lazy::force(&ANALYSIS_TABLE);
        let (tree, accept, error) = generate_tree(lexer, *analysis_table);
        println!("{}", tree.borrow().generate_mermaid());
        assert_eq!(accept, false);
        assert_eq!(error, true);
    }
}
