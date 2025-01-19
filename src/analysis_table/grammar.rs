use std::collections::{HashMap, HashSet};
use core::fmt::Display;

use crate::{common::types::{IdToken, Token}, parser::Lexem};

use super::NonTerminal;

/// Représente un lexem qui a été prsé de la grammaire
#[derive(Clone, Debug, PartialEq, Eq)]
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

impl Into<Lexem> for ParsedLexem {
    fn into(self) -> Lexem {
        self.lexem
    }
}

/// Représente une règle d'une grammaire
#[derive(PartialEq, Eq)]
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
    pub non_terminal_lexems: HashSet<ParsedLexem>,

    /// Set des non terminaux produisant le mot vide
    empty_word_producers: HashSet<NonTerminal>,

    /// Permet de savoir si la liste des non terminaux produisant le mot vide a été calculée
    empty_word_producers_computed: bool,

    /// Liste des premiers pour les non terminaux
    firsts: HashMap<NonTerminal, HashSet<Token>>,

    /// Permet de savoir si la liste des premiers a été calculée
    firsts_computed: bool,

    /// Liste des suivants pour les non terminaux
    follows: HashMap<NonTerminal, HashSet<Token>>,

    /// Permet de savoir si la liste des suivants a été calculée
    follows_computed: bool,

}

/// Crée le ParsedLexem correspondant à un lexem terminal
macro_rules! terminal {
    ($name: expr, $token: expr) => {
        ParsedLexem {name: $name.to_string(), lexem: Lexem::Terminal($token)}
    };
}

/// Convertit un Lexem en NonTerminal. Panic si le lexem est terminal
macro_rules! lexem_to_nonterm {
    ($lexem: expr) => {
        match $lexem {
            Lexem::Terminal(t) => panic!("Cannot convert terminal lexem {} to non terminal", t),
            Lexem::NonTerminal(nt) => nt,
        }
    };
}

impl Grammar {

    /// Crée une nouvelle grammaire vide
    pub fn new() -> Self {
        return Grammar {
            rules: Vec::new(),
            non_terminal_lexems: HashSet::new(),
            empty_word_producers: HashSet::new(),
            empty_word_producers_computed: false,
            firsts: HashMap::new(),
            firsts_computed: false,
            follows: HashMap::new(),
            follows_computed: false,
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
            "in" => terminal!("in", Token::In),
            "def" => terminal!("def", Token::Def),
            "return" => terminal!("return", Token::Return),
            "print" => terminal!("print", Token::Print),

            // Le lexem est un non terminal
            name => ParsedLexem {name: String::from(name), lexem: Lexem::NonTerminal(NonTerminal::from(String::from(name)))},
        }
    }

    /// Ajoute une règle à la grammaire en utilisant les lexems 
    /// déjà parsés ou en en créant des nouveaux si besoin
    /// 
    /// Les producteurs de mot vide, premiers et suivants doivent être recalculés après l'appel à cette fonction
    pub fn create_rule(&mut self, start: &str, products: Vec<String>) {
        let r = Rule {
            id: self.rules.len(),
            start: self.get_lexem(start),
            production: products.iter().map(|l| self.get_lexem(l)).collect(),
        };
        self.rules.push(r);

        self.empty_word_producers_computed = false;
        self.firsts_computed = false;
        self.follows_computed = false;
    }

    /// Renvois tous les non-terminaux produisant le mod vide, en les calculant si besoin.
    pub fn empty_word_producers(&mut self) -> &HashSet<NonTerminal> {

        if !self.empty_word_producers_computed {
            self.compute_empty_word_producers();
        }

        &self.empty_word_producers
    }

    /// Calcule la liste des non-terminaux produisant le mot vide
    pub fn compute_empty_word_producers(&mut self) {

        // Set des non terminaux produisant le mot vid
        let mut producers: HashSet<NonTerminal> = HashSet::new();

        loop {

            let mut changed = false;

            for rule in &self.rules {

                let start_lexem = match &rule.start.lexem {
                    Lexem::Terminal(_) => panic!(),
                    Lexem::NonTerminal(nt) => nt
                };

                // Skip rules that starts with an empty word producer
                if producers.contains(start_lexem) {
                    continue;
                }

                // Check if production is comprised of empty word producers only
                let produces_empty_word = rule.production.iter()
                    .all(|lexem| {
                        match &lexem.lexem {
                            Lexem::Terminal(_) => false,
                            Lexem::NonTerminal(nt) => producers.contains(nt),
                        }
                    });

                if produces_empty_word {
                    changed = true;
                    producers.insert(start_lexem.clone());
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
            .filter_map(|entry| {
                match &entry.lexem {
                    Lexem::NonTerminal(nt) if producers.contains(&nt) => Some(nt.clone()),
                    _ => None,
                }
            })
            .collect();

        self.empty_word_producers = producers;
    }

    /// Renvoie les premiers de tous les non terminaux
    pub fn firsts(&mut self) -> &HashMap<NonTerminal, HashSet<Token>> {
        if !self.firsts_computed {
            self.compute_firsts();
        }
        &self.firsts
    }

    /// Renvoie les premiers pour un non terminal, sans les calculer si besoin
    /// Panique si les premiers n'ont pas déjà été calculés
    pub fn get_firsts_unmut(&self, lexem: &Lexem) -> Option<&HashSet<Token>> {
        if !self.firsts_computed {
            panic!("Trying to get firsts without computing them")
        }

        match lexem {
            Lexem::Terminal(_) => panic!("Trying to get firsts for a terminal"),
            Lexem::NonTerminal(nt) => self.firsts.get(nt),
        }
    }

    /// Calcule Premier(word) avec word un mot composé de terminaux et de non terminaux
    /// Panique si les premiers et les producteurs de mot vide ne sont pas précalculés
    pub fn get_word_firsts_unmut(&self, word: &[ParsedLexem]) -> HashSet<Token> {

        let mut firsts: HashSet<Token> = HashSet::new();

        for lexem in word {

            if let Lexem::Terminal(token) = &lexem.lexem {
                firsts.insert(token.clone());
                break;

            } else {
                firsts.extend(
                    self.get_firsts_unmut(&lexem.lexem)
                        .unwrap_or(&HashSet::new())
                        .iter()
                        .map(|t| t.clone())
                );
                if !self.produces_empty_word_unmut(&lexem.lexem) {
                    break;
                }
            }
        }

        return firsts;
    }

    /// Calcule les premiers pour tous les non terminaux de la grammaire
    pub fn compute_firsts(&mut self) {

        // Initialise des ensembles vides pour les premiers de chaque non terminal
        let mut firsts: HashMap<NonTerminal, HashSet<Token>> = HashMap::new();

        // Calcule les producteurs de mots vide si ce n'est pas déjà fait
        // Nécessaire pour utiliser Self::produces_empty_word_unmut
        if !self.empty_word_producers_computed {
            self.compute_empty_word_producers();
        }

        loop {
            let mut changed = false;

            for rule in &self.rules {

                let start_nt = lexem_to_nonterm!(&rule.start.lexem);

                for lexem in &rule.production {

                    if let Lexem::Terminal(token) = &lexem.lexem {
                        changed = firsts.entry(start_nt.clone()).or_default().insert(token.clone()) || changed;
                        break;
                    
                    } else if let Lexem::NonTerminal(nt) = &lexem.lexem {
                        
                        let firsts_copy: Vec<Token> = firsts.entry(nt.clone()).or_default().iter().map(|t| t.clone()).collect();
                        
                        for first in firsts_copy {
                            changed = firsts.entry(start_nt.clone()).or_default().insert(first) || changed;
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

    /// Détermine si un Lexem peut produire le mot vide, sans calculer les producteurs de
    /// mot vide. Si les producteurs de mot vide n'ont pas été calculés, panique.
    pub fn produces_empty_word_unmut(&self, lexem: &Lexem) -> bool {
        if !self.empty_word_producers_computed {
            panic!("Trying to get empty word producers without computing it first");
        }
        match lexem {
            Lexem::Terminal(_) => false,
            Lexem::NonTerminal(nt) => self.empty_word_producers.contains(nt),
        }
    }

    /// Détermine si un mot peut produire le mot vide, sans calculer les producteurs de
    /// mot vide. Si les producteurs de mot vide n'ont pas été calculés, panique.
    pub fn word_produces_empty_word_unmut(&self, word: &[ParsedLexem]) -> bool {
        if !self.empty_word_producers_computed {
            panic!("Trying to get empty word producers without computing it first");
        }

        for lexem in word {
            match &lexem.lexem {
                Lexem::Terminal(_) => return false,
                Lexem::NonTerminal(nt) if !self.empty_word_producers.contains(nt) => return false,
                _ => ()
            }
        }
        return true;
    }

    /// Renvoie les suivants de la grammaire
    pub fn follows(&mut self) -> &HashMap<NonTerminal, HashSet<Token>> {
        if !self.follows_computed {
            self.compute_follows();
        }
        &self.follows
    }

    /// Calcule les suivants pour tous les non terminaux de la grammaire
    pub fn compute_follows(&mut self) {

        // Initialise des ensembles vides pour les suivants de chaque non terminal
        let mut follows: HashMap<NonTerminal, HashSet<Token>> = HashMap::new();

        for lexem in &self.non_terminal_lexems {
            let nt: NonTerminal = lexem_to_nonterm!(lexem.lexem.clone());
            follows.insert(nt, HashSet::new());
        }

        // Préalcule les premiers si besoin
        if !self.firsts_computed {
            self.compute_firsts();
        }

        // Préalcule les producteurs de mot vide si besoin
        if !self.empty_word_producers_computed {
            self.compute_empty_word_producers();
        }
        
        
        // Initialise l'axiome. On suppose que le "$" est représenté par le token EOF
        follows.entry(NonTerminal::File).or_default().insert(Token::EOF);


        loop {

            let mut changed = false;

            for rule in &self.rules {
    
                let mut current_lexem_index: usize = 0;
    
                for lexem in &rule.production {
    
                    let current_nonterm = match &lexem.lexem {
                        Lexem::Terminal(_) => {
                            current_lexem_index += 1;
                            continue;
                        },
                        Lexem::NonTerminal(nt) => nt.clone(),
                    };
    
                    // Le lexem actuel n'est pas terminal, on ajoute les premiers de la suite du mot aux suivants de ce lexem
                    for t in self.get_word_firsts_unmut(&rule.production[current_lexem_index + 1 ..]) {
                        changed = follows.entry(current_nonterm.clone()).or_default().insert(t.clone()) || changed;
                    }

                    // Si la suite du mot peut devenir le mot vide, on ajoute les suivants
                    // du terminal à gauche de la règle
                    if self.word_produces_empty_word_unmut(&rule.production[current_lexem_index + 1 ..]) {
                        
                        let follows_copy: Vec<Token> = follows.entry(lexem_to_nonterm!(&rule.start.lexem).clone()).or_default().iter().cloned().collect();
                        for t in follows_copy {
                            changed = follows.entry(current_nonterm.clone()).or_default().insert(t.clone()) || changed;
                        }

                    }

                    current_lexem_index += 1;

                }
            }

            if !changed {
                break;
            }
        }

        self.follows_computed = true;
        self.follows = follows;
    }

    /// Renvoie un HashSet des suivants d'un non terminal.
    /// Si les suivants n'ont pas encore été calculés, panique
    pub fn get_follows_unmut(&self, non_terminal: &ParsedLexem) -> &HashSet<Token> {
        let Lexem::NonTerminal(nt) = &non_terminal.lexem else {
            panic!("Trying to get follows of a terminal");
        };
        return &self.follows[nt]
    }

}


