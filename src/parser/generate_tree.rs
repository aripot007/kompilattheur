use colored::Colorize;

use crate::{analysis_table::NonTerminal, common::{diagnostic::{Diagnostic, DiagnosticGravity}, types::{file_element::{self, file_element_from}, FileElement}}, lexer::Lexer};
use super::lexem::Lexem;
use crate::analysis_table::AnalysisTable;
use crate::common::types::{Node, Token};
use std::cell::RefCell;
use std::rc::Rc;

pub fn generate_tree(mut lexer: Lexer, analysis_table: &AnalysisTable) -> (Rc<RefCell<Node<FileElement<Lexem>>>>, bool, bool) {
    
    let tree: Rc<RefCell<Node<FileElement<Lexem>>>> = Node::new(
        FileElement { line: 0, start_char: 0, len: 0, element: Lexem::NonTerminal(NonTerminal::File) }
    );
    let mut stack: Vec<Rc<RefCell<Node<FileElement<Lexem>>>>> = vec![tree.clone()];
    let mut error = false;
    let mut accept = false;
    let mut is_acceptable = true;
    let mut input = lexer.next().unwrap_or(file_element::EOF);

    while !accept {
        if error {
            is_acceptable = false;
        }
        //println!("Stack: {:?}, Input: {}", stack, input);
        let x = stack.pop();
        //println!("Tree: {:?}", tree.borrow());
        //println!("Node: {:?}", x);
        match x {
            Some(node) => {
                let file_elem = node.borrow_mut().get_value().clone();
                match file_elem.element {
                    Lexem::Terminal(token) => {
                        if token == Token::Newline && input.element == Token::EOF {
                            Diagnostic::new(
                                DiagnosticGravity::Warning,
                                "ParserEndOfFileWarning :".to_string(),
                                input.line,
                                input.line,
                                input.start_char,
                                if input.len > 0 {
                                    input.start_char + (input.len - 1)
                                } else {
                                    input.start_char
                                },
                                "No Newline at the end of the file".to_string(),
                            ).display();
                            node.borrow_mut().set_value(file_element_from!(input, Lexem::Terminal(input.element.clone())));
                            input = lexer.next().unwrap_or(file_element::EOF);
                            error = false;
                        } 
                        else if token.is_same_type(&input.element) {
                            //println!("Input: {:?}", input);
                            node.borrow_mut().set_value(file_element_from!(input, Lexem::Terminal(input.element.clone())));
                            input = lexer.next().unwrap_or(file_element::EOF);
                            error = false;
                        } else {
                            if error {
                                //println!("2: Error: {} Stack: {}", input.element, token);
                                if token == Token::Newline {
                                    while input.element != Token::Newline && input.element != Token::EOF {
                                        input = lexer.next().unwrap_or(file_element::EOF);
                                    }
                                    error = false;
                                    input = lexer.next().unwrap_or(file_element::EOF);
                                    //println!("3: Error: {} Stack: {}", input.element, token);
                                    continue;
                                }
                                if stack.is_empty() {
                                    break;
                                }
                                input = lexer.next().unwrap_or(file_element::EOF);
                                continue;
                            }
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
                                if input.len > 0 {
                                    input.start_char + input.len - 1
                                } else {
                                    input.start_char
                                },
                                format!(
                                    "Expected {} but got {}",
                                    token.to_string().truecolor(255, 0, 0),
                                    input.element.to_string().truecolor(255, 0, 0)
                                )
                                .to_string(),
                            )
                            .display();
                            // println!("Error: {token:?} != {}", input.element);
                        }
                    }
                    Lexem::NonTerminal(id) => {
                        let entry = analysis_table.get(&id, &input.element);
                        match entry {
                            Some(lexems) => {
                                error = false;
                                for lexem in lexems.iter().rev() {
                                    let new_node = Node::new(
                                        FileElement {
                                            len: 0,
                                            line: 0,
                                            start_char: 0,
                                            element: (*lexem).clone(),
                                        }
                                    );
                                    stack.push(new_node.clone());
                                    node.borrow_mut().insert_child(&node, 0, new_node.clone());
                                }
                            }
                            None => {
                                if error {
                                    //println!("Error: {} Stack: {}", input.element, Lexem::NonTerminal(id));
                                    //input = lexer.next().unwrap_or(file_element::EOF);
                                    continue;
                                }
                                error = true;
                                let expected_tokens = analysis_table.get_expected_tokens(&id);
                                let expected_tokens = expected_tokens
                                    .iter()
                                    .map(|x| match x {
                                        Token::Identifier(_) => "Identifier"
                                            .to_string()
                                            .truecolor(255, 0, 0)
                                            .to_string(),
                                        Token::Integer(_) => {
                                            "Integer".to_string().truecolor(255, 0, 0).to_string()
                                        }
                                        Token::String(_) => {
                                            "String".to_string().truecolor(255, 0, 0).to_string()
                                        }
                                        _ => x.to_string().truecolor(255, 0, 0).to_string(),
                                    })
                                    .collect::<Vec<String>>()
                                    .join(", ");
                                Diagnostic::new(
                                    DiagnosticGravity::Error,
                                    "ParserTableError :".to_string(),
                                    input.line,
                                    input.line,
                                    input.start_char,
                                    if input.len > 0 {
                                        input.start_char + input.len - 1
                                    } else {
                                        input.start_char
                                    },
                                    format!(
                                        "Expected {} but got {}",
                                        expected_tokens,
                                        input.element.to_string().truecolor(255, 0, 0)
                                    )
                                    .to_string(),
                                )
                                .display();
                                input = lexer.next().unwrap_or(file_element::EOF);
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
                        input.start_char + input.len - 1,
                        "Stack is empty and input is not EOF".to_string(),
                    )
                    .display();
                    println!("Error: Stack is empty and input is not EOF");
                }
                accept = true;
            }
        }
    }

    // Finish lexical analysis before returning
    if error {
        while let Some(_) = lexer.next() {}
    }
    if lexer.get_nb_errors() > 0 {
        is_acceptable = false;
    }
    if !is_acceptable {
        accept = false;
        error = true;
    }

    return (tree, accept, error);
}

#[cfg(test)]
mod tests {

    use crate::analysis_table::setup_analysis_table;
    use std::path::PathBuf;

    use super::*;

    use once_cell::sync::Lazy;

    static ANALYSIS_TABLE: Lazy<&AnalysisTable> =
        Lazy::new(|| setup_analysis_table(Some(&PathBuf::from("grammaire.txt"))));

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
