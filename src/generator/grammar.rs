use std::collections::{HashMap, HashSet};
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

    /// Lexemes non terminaux
    pub non_terminal_lexems: HashMap<String, ParsedLexem>,
    next_non_terminal_id: usize,

    /// Non terminaux produisant le mot vide
    empty_word_producers: Vec<ParsedLexem>,

    /// Tableau contenant true pour les non terminaux produisant le mot vide
    empty_word_producers_ids: Vec<bool>,

    /// Permet de savoir si la liste des non terminaux produisant le mot vide a été calculée
    empty_word_producers_computed: bool,

    /// Liste des premiers pour les non terminaux
    firsts: Vec<HashSet<Token>>,

    firsts_computed: bool,

}

/// Crée le ParsedLexem correspondant à un lexem terminal
macro_rules! terminal {
    ($name: expr, $token: expr) => {
        ParsedLexem {name: $name.to_string(), lexem: Lexem::Terminal($token)}
    };
}

/// Renvoie l'id d'un ParsedLexem non terminal
/// Panic si le ParsedLexem n'est pas un non terminal
macro_rules! non_terminal_id {
    ($parsed_lexem: expr) => {
        match $parsed_lexem.lexem {
            Lexem::Terminal(_) => panic!("Trying to get terminal id from non-terminal ParsedLexem"),
            Lexem::NonTerminal(id) => id,
        }
    };
}

impl Grammar {

    pub fn new() -> Self {
        return Grammar {
            rules: Vec::new(),
            next_non_terminal_id: 0,
            non_terminal_lexems: HashMap::new(),
            empty_word_producers: Vec::new(),
            empty_word_producers_ids: Vec::new(),
            empty_word_producers_computed: false,
            firsts: Vec::new(),
            firsts_computed: false,
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

    pub fn empty_word_producers(&mut self) -> &Vec<ParsedLexem> {

        if !self.empty_word_producers_computed {
            self.compute_empty_word_producers();
        }

        &self.empty_word_producers
    }

    /// Calcule la liste des non-terminaux produisant le mot vide
    fn compute_empty_word_producers(&mut self) {

        let nb_non_terminal = self.non_terminal_lexems.len();

        // Tabeau de booléen des non terminaux produisant le mot vid
        let mut producers: Vec<bool> = vec![false; nb_non_terminal];

        loop {

            let mut changed = false;

            for rule in &self.rules {

                let start_lexem_id = non_terminal_id!(rule.start);

                // Skip rules that starts with an empty word producer
                if producers[start_lexem_id] {
                    continue;
                }

                // Check if production is comprised of empty word producers only
                let produces_empty_word = rule.production.iter()
                    .all(|lexem| {
                        match lexem.lexem {
                            Lexem::Terminal(_) => false,
                            Lexem::NonTerminal(id) => producers[id],
                        }
                    });

                if produces_empty_word {
                    changed = true;
                    producers[start_lexem_id] = true;
                }
            }

            if !changed {
                break;
            }

        }
        
        self.empty_word_producers_computed = true;

        // Récupère les non terminaux correspondant
        self.empty_word_producers = self.non_terminal_lexems
            .iter()
            .filter(|entry| producers[non_terminal_id!(entry.1)])
            .map(|entry| entry.1.clone())
            .collect();

        self.empty_word_producers_ids = producers;
    }

    /// Renvoie les premiers de tous les non terminaux
    pub fn firsts(&mut self) -> &Vec<HashSet<Token>> {
        if !self.firsts_computed {
            self.compute_firsts();
        }
        &self.firsts
    }

    /// Renvoie les premiers pour un non terminal
    pub fn get_firsts(&mut self, lexem: &Lexem) -> &HashSet<Token> {
        if !self.firsts_computed {
            self.compute_firsts();
        }

        match lexem {
            Lexem::Terminal(_) => panic!("Trying to get firsts for a terminal"),
            Lexem::NonTerminal(id) => &self.firsts[*id],
        }
    }

    fn compute_firsts(&mut self) {

        // Initialise des ensembles vides pour les premiers de chaque non terminal
        let mut firsts: Vec<HashSet<Token>> = vec![HashSet::new(); self.non_terminal_lexems.len()];

        // Calcule les producteurs de mots vide si ce n'est pas déjà fait
        // Nécessaire pour utiliser Self::produces_empty_word_unmut
        if !self.empty_word_producers_computed {
            self.compute_empty_word_producers();
        }

        loop {
            let mut changed = false;

            for rule in &self.rules {

                let start_id = non_terminal_id!(rule.start);

                for lexem in &rule.production {

                    if let Lexem::Terminal(token) = &lexem.lexem {
                        changed = firsts[start_id].insert(token.clone()) || changed;
                        break;
                    
                    } else if let Lexem::NonTerminal(id) = &lexem.lexem {
                        
                        let firsts_copy: Vec<Token> = firsts[*id].iter().map(|t| t.clone()).collect();
                        
                        for first in firsts_copy {
                            changed = firsts[start_id].insert(first) || changed;
                        }

                        if !self.produces_empty_word_unmut(&lexem.lexem) {
                            break;
                        }
                    }
                }
            }

            if !changed {
                break;
            }
        }

        self.firsts = firsts;
        self.firsts_computed = true;
    }

    /// Détermine si un Lexem peut produire le mot vide.
    /// Calcule les producteurs de mot vide si ils n'ont pas été calculés
    pub fn produces_empty_word(&mut self, lexem: &Lexem) -> bool {
        if !self.empty_word_producers_computed {
            self.compute_empty_word_producers();
        }
        match lexem {
            Lexem::Terminal(_) => false,
            Lexem::NonTerminal(id) => *self.empty_word_producers_ids.get(*id).unwrap_or(&false),
        }
    }

    /// Détermine si un Lexem peut produire le mot vide, sans calculer les producteurs de
    /// mot vide. Si les producteurs de mot vide n'ont pas été calculés, panique.
    pub fn produces_empty_word_unmut(&self, lexem: &Lexem) -> bool {
        if !self.empty_word_producers_computed {
            panic!("Trying to get empty word producers without computing it first");
        }
        match lexem {
            Lexem::Terminal(_) => false,
            Lexem::NonTerminal(id) => *self.empty_word_producers_ids.get(*id).unwrap_or(&false),
        }
    }

}


