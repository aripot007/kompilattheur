use crate::common::types::token::Token;
use crate::parser::lexem::Lexem;
use core::fmt;
use std::collections::HashMap;
use std::fmt::Display;
use std::mem::{discriminant, Discriminant};

use super::grammar::{Grammar, ParsedLexem};

/// Représente une table d'analyse pour une grammaire
pub struct AnalysisTable {
    /// La table d'analyse. Pour chaque non terminal, contient une HashMap des tokens lu et de la production de la règle correspondante
    pub table: Vec<HashMap<Discriminant<Token>, Vec<Lexem>>>,
    
    pub non_terminal_names: Vec<String>,

    // Permet de map un discriminant à un token pour le debug
    discriminant_tokens: HashMap<Discriminant<Token>, Token>,
}


/// Renvoie la représentation générique d'un token, ie le nom sans les informations du token.
/// 
/// ```
/// let t = Token::Add;
/// assert_eq!(t.repr(), generic_token_repr!(t));
/// assert_eq!("<string>", generic_token_repr!(Token::String("Hello")));
/// assert_eq!("<integer>", generic_token_repr!(Token::integer(42)));
/// assert_eq!("<ident>", generic_token_repr!(Token::Identifier(IdToken {42})));
/// ```
macro_rules! generic_token_repr {
    ($token: expr) => {
        match $token {
            Token::Identifier(_) => String::from("<ident>"),
            Token::String(_) => String::from("<string>"),
            Token::Integer(_) => String::from("<integer>"),
            _ => $token.repr(),
        }
    };
}

impl AnalysisTable {

    pub fn get(&self, id: &usize, token: &Token) -> Option<&Vec<Lexem>> {
        self.table[*id].get(&discriminant(token))
    }

    /// Essaie d'ajouter une règle une case de la table d'analyse.
    /// Si une règle est déjà présente (grammaire non LL(1)), panique
    fn try_set(&mut self, left_non_terminal: &ParsedLexem, token: &Token, production: &Vec<ParsedLexem>) {
        
        let left_lexem_id = match &left_non_terminal.lexem {
            Lexem::Terminal(t) => panic!("Trying to fill analysis table with the terminal {} on the left of a rule", t),
            Lexem::NonTerminal(id) => id,
        };

        // Convert ParsedLexem to Lexem
        let production: Vec<Lexem> = production.iter().map(|l| l.lexem.clone()).collect();

        match self.table[*left_lexem_id].insert(discriminant(token), production) {
            Some(prod) => todo!(),
            None => (),
        };

        // Save the token associated to the discriminant
        self.discriminant_tokens.insert(discriminant(token), token.clone());
    }
}

impl From<&Grammar> for AnalysisTable {

    fn from(grammar: &Grammar) -> Self {
        
        let nb_non_terminal = grammar.non_terminal_lexems.len();

        // Initialize an empty analysis table

        let analysis_table: Vec<HashMap<Discriminant<Token>, Vec<Lexem>>> = vec![HashMap::new(); nb_non_terminal];

        let mut non_terminal_names: Vec<String> = vec![String::new(); grammar.non_terminal_lexems.len()];

        for (name, lexem) in grammar.non_terminal_lexems.iter() {
            match lexem.lexem {
                Lexem::Terminal(_) => continue,
                Lexem::NonTerminal(id) => non_terminal_names[id] = name.clone(),
            }
        }

        let mut table = AnalysisTable {
            table: analysis_table,
            non_terminal_names,
            discriminant_tokens: HashMap::new(),
        };

        // Try to fill the table

        for rule in &grammar.rules {
            for p in &grammar.get_word_firsts_unmut(&rule.production) {
                table.try_set(&rule.start, p, &rule.production);
            }

            if grammar.word_produces_empty_word_unmut(&rule.production) {
                for s in grammar.get_follows_unmut(&rule.start) {
                    table.try_set(&rule.start, s, &rule.production);
                }
            }
        }

        return table;
    }

}

// Implémentation de Display avec plusieurs formats
// Cf https://play.rust-lang.org/?version=stable&mode=debug&edition=2015&gist=bf6bc18c5f709ab237d7948d0dfb5644
// pour un exemple plus lisible
impl AnalysisTable {

    /// Formate la table d'analyse dans un format lisible en texte clair
    pub fn display_plain<'a>(&'a self) -> impl fmt::Display + 'a {

        struct PlainAnalysisTable<'a>(&'a AnalysisTable);
        impl<'a> fmt::Display for PlainAnalysisTable<'a> {

            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

                // Map discriminants to an index to keep correct order in the table rows
                let mut discriminant_ids: HashMap<Discriminant<Token>, usize> = HashMap::new();
        
                let mut next_id = 0;
                for d in self.0.discriminant_tokens.keys() {
                    if !discriminant_ids.contains_key(d) {
                        discriminant_ids.insert(d.clone(), next_id);
                        next_id += 1;
                    }
                }
        
                let nb_discriminants = next_id;
        
                // Construct strings in cell while keeping track of the max len for each column
                let mut column_sizes: Vec<usize> = vec![0; nb_discriminants];
                let mut discriminant_names: Vec<String> = vec![String::new(); nb_discriminants];
        
                // Initialize column size with non terminal name
                for (discr, token) in self.0.discriminant_tokens.iter() {
        
                    let name = generic_token_repr!(token);
        
                    let discr_id = discriminant_ids[discr];
                    column_sizes[discr_id] = name.len();
                    discriminant_names[discr_id] = name;
                }
        
                // Fill table while keeping track of column max len
                let nb_non_terminals = self.0.table.len();
                let mut str_table: Vec<Vec<String>> = vec![vec![String::new(); nb_discriminants]; nb_non_terminals];
        
                for i in 0 .. nb_non_terminals {
                    for (discr, word) in &self.0.table[i] {
        
                        let discr_id = discriminant_ids[discr];
        
                        // Compute word string
                        let word_str: String = word.iter()
                            .map(|lexem| {
                                match lexem {
                                    Lexem::NonTerminal(id) => self.0.non_terminal_names[*id].clone(),
                                    Lexem::Terminal(token) => generic_token_repr!(token),
                                }
                            })
                            .collect();
        
                        if word_str.len() > column_sizes[discr_id] {
                            column_sizes[discr_id] = word_str.len();
                        }
        
                        str_table[i][discr_id] = word_str;
                    }
                }
        
                // Add 2 spaces for the padding
                column_sizes = column_sizes.iter().map(|s| s + 2).collect();
        
                // Starting non-terminal column size
                let left_lexem_max_size = self.0.non_terminal_names
                    .iter()
                    .map(String::len)
                    .max()
                    .expect("Trying to display empty analysis table");
        
                // Add padding
                let left_col_size = left_lexem_max_size + 2;
        
                // Construct separator string
                let mut sep_str: String = format!("+{:->width$}", "+", width=left_col_size+1);
                
                for size in &column_sizes {
                    // Add 1 space for the "+" character to the column size
                    sep_str += &format!("{:->width$}", "+", width=size+1);
                }
                sep_str += "\n";
        
                
                // Print header
                write!(f, "{}", sep_str)?;
        
                write!(f, "+{:>width$}", "+", width=left_col_size+1)?;
                for (i, name) in discriminant_names.iter().enumerate() {
                    write!(f, "{:^width$}+", name, width=column_sizes[i])?;
                }
                write!(f, "\n")?;
        
                write!(f, "{}", sep_str)?;
        
                // Print table
                for i in 0..nb_non_terminals {
        
                    write!(f, "|{:^width$}|", self.0.non_terminal_names[i], width=left_col_size)?;
        
                    for j in 0..nb_discriminants {
                        write!(f, "{:^width$}|", str_table[i][j], width=column_sizes[j])?;
                    }
        
                    write!(f, "\n{}", sep_str)?;
                }
        
                Ok(())
            }
        }
        PlainAnalysisTable(self)
    }
    
    /// Formate la table d'analyse dans un format markdown
    pub fn display_markdown<'a>(&'a self) -> impl fmt::Display + 'a {

        /// Escape markdown characters that could interfer with a table
        macro_rules! escape_markdown {
            ($s: expr) => {
                $s.replace("|", "\\|")
                  .replace("\\", "\\\\")
                  .replace("(", "\\(")
                  .replace(")", "\\)")
                  .replace("[", "\\[")
                  .replace("]", "\\]")
                  .replace("<", "\\<")
                  .replace(">", "\\>")
            };
        }

        struct MarkdownAnalysisTable<'a>(&'a AnalysisTable);
        impl<'a> fmt::Display for MarkdownAnalysisTable<'a> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

                // Map discriminants to an index to keep correct order in the table rows
                let mut discriminant_ids: HashMap<Discriminant<Token>, usize> = HashMap::new();

                // Map discriminants to their names
                let mut discriminant_names: Vec<String> = Vec::new();

                for (d, token) in self.0.discriminant_tokens.iter() {
                    if !discriminant_ids.contains_key(d) {
                        discriminant_ids.insert(d.clone(), discriminant_names.len());
                        discriminant_names.push(generic_token_repr!(token));
                    }
                }

                let nb_discriminants = discriminant_names.len();


                // An auxiliary table is necessary to keep the correct order for the rows
                let nb_non_terminals = self.0.table.len();
                let mut str_table: Vec<Vec<String>> = vec![vec![String::new(); nb_discriminants]; nb_non_terminals];

                for i in 0 .. nb_non_terminals {
                    for (discr, word) in &self.0.table[i] {

                        let discr_id = discriminant_ids[discr];

                        // Compute word string
                        let word_str: String = word.iter()
                            .map(|lexem| {
                                match lexem {
                                    Lexem::NonTerminal(id) => self.0.non_terminal_names[*id].clone(),
                                    Lexem::Terminal(token) => generic_token_repr!(token),
                                }
                            })
                            .collect();
                            

                        // Escape markdown
                        let word_str = escape_markdown!(word_str);

                        str_table[i][discr_id] = word_str;
                    }
                }

                // Print header
                write!(f, "| |")?;
                for (i, name) in discriminant_names.iter().enumerate() {
                    write!(f, "{}|", escape_markdown!(name))?;
                }
                write!(f, "\n")?;

                // Header separator 
                write!(f, "|-|")?;
                for _ in 0..nb_discriminants {
                    write!(f, "-|")?;
                }
                write!(f, "\n")?;

                // Print table
                for i in 0..nb_non_terminals {

                    write!(f, "|{}|", escape_markdown!(self.0.non_terminal_names[i]))?;

                    for j in 0..nb_discriminants {
                        write!(f, "{}|", str_table[i][j])?;
                    }

                    write!(f, "\n")?;
                }
                Ok(())
            }
        }
        MarkdownAnalysisTable(self)
    }
}

impl Display for AnalysisTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_plain().fmt(f)
    }
}
