use std::collections::HashMap;
use core::fmt::Display;

use crate::{common::types::token::{IdToken, NumToken, Token}, parser::lexem::Lexem};

/// Représente un lexem qui a été prsé de la grammaire
#[derive(Clone, Debug)]
pub struct ParsedLexem {
    /// Nom du lexem dans le fichier de la grammaire
    pub name: String,

    /// type Lexem utilisé dans le reste du compilateur
    pub lexem: Lexem,
}

impl Display for ParsedLexem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

/// Représente une règle d'une grammaire
pub struct Rule {
    pub id: usize,
    pub start: ParsedLexem,
    pub production: Vec<ParsedLexem>
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}) {} -> ", self.id, self.start)?;

        for l in &self.production {
            write!(f, "{}", l)?;
        }

        Ok(())
    }
}

/// Représente une grammaire
pub struct Grammar {

    /// Règles de la grammaire
    pub rules: Vec<Rule>,
    pub non_terminal_lexems: HashMap<String, ParsedLexem>,
    next_non_terminal_id: usize,

}

/// Crée le ParsedLexem correspondant à un lexem terminal
macro_rules! terminal {
    ($name: expr, $token: expr) => {
        ParsedLexem {name: $name.to_string(), lexem: Lexem::Terminal($token)}
    };
}

impl Grammar {

    pub fn new() -> Self {
        return Grammar {
            rules: Vec::new(),
            next_non_terminal_id: 0,
            non_terminal_lexems: HashMap::new(),
        }
    }

    /// Récupère le ParsedLexem correspondant au nom donné, le crée et l'ajoute à la grammaire si besoin
    pub fn get_lexem(&mut self, name: &str) -> ParsedLexem {
        match name {
            // Match les lexem terminaux

            // Tokens spéciaux
            "EOF" => terminal!("EOF", Token::EOF),
            "NEWLINE" => terminal!("NEWLINE", Token::Newline),
            "BEGIN" => terminal!("BEGIN", Token::Begin),
            "END" => terminal!("END", Token::End),

            // Operators
            "+" => terminal!("+", Token::Add),
            "-" => terminal!("-", Token::Sub),
            "*" => terminal!("*", Token::Mult),
            "//" => terminal!("//", Token::Div),
            "%" => terminal!("%", Token::Mod),
            "=" => terminal!("=", Token::Assign),

            // Boolean
            "==" => terminal!("==", Token::Equal),
            "!=" => terminal!("!=", Token::NotEqual),
            ">" => terminal!(">", Token::Greater),
            ">=" => terminal!(">=", Token::GreaterEq),
            "<" => terminal!("<", Token::Less),
            "<=" => terminal!("<=", Token::LessEq),

            // Program structure
            ":" => terminal!(":", Token::Sep),
            "," => terminal!(",", Token::Comma),
            "(" => terminal!("(", Token::OpenParenthesis),
            ")" => terminal!(")", Token::CloseParenthesis),
            "[" => terminal!("[", Token::OpenBracket),
            "]" => terminal!("]", Token::CloseBracket),

            "<integer>" => terminal!("<integer>", Token::integer(0)),
            "<string>" => terminal!("<string>", Token::String(String::new())),
            "<ident>" => terminal!("<ident>", Token::Identifier(IdToken {id: 0})),

            // Keywords
            "True" => terminal!("True", Token::True),
            "False" => terminal!("False", Token::False),
            "None" => terminal!("None", Token::None),
            "or" => terminal!("or", Token::Or),
            "and" => terminal!("and", Token::And),
            "not" => terminal!("not", Token::Not),
            "if" => terminal!("if", Token::If),
            "else" => terminal!("else", Token::Else),
            "for" => terminal!("for", Token::For),
            "def" => terminal!("def", Token::Def),
            "return" => terminal!("return", Token::Return),
            "print" => terminal!("print", Token::Print),

            // Le lexem est un non terminal
            name => {
                let name = String::from(name);
                self.non_terminal_lexems
                    .entry(name.clone())
                    .or_insert_with(|| {
                        let id = self.next_non_terminal_id;
                        self.next_non_terminal_id += 1;
                        ParsedLexem {
                            name: name.clone(),
                            lexem: Lexem::NonTerminal(id),
                        }
                    })
                    .clone()
            }
        }
    }


    /// Ajoute une règle à la grammaire en utilisant les lexems 
    /// déjà parsés ou en en créant des nouveaux si besoin
    pub fn create_rule(&mut self, start: &str, products: Vec<String>) {
        let r = Rule {
            id: self.rules.len(),
            start: self.get_lexem(start),
            production: products.iter().map(|l| self.get_lexem(l)).collect(),
        };
        self.rules.push(r);
    }

}


