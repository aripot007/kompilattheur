mod markdown;
mod plaintext;
use super::analysis_table::AnalysisTable;
use std::{collections::HashMap, mem::Discriminant};
use crate::common::types::token::Token;
use crate::parser::lexem::Lexem;

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

pub (super) use generic_token_repr;

/// Construct a `Vec<Vec<String>>` with the text that should be in each cell of the table.
/// Also returns a `discriminant_id` HashMap that maps discriminants to their column id, and
/// a `discriminants_names` Vec that maps a discriminant id to their display name.
/// 
/// ```text
/// let table, discr_ids, discr_names = construct_string_table(&analysis_table);
/// 
/// println!("Terminals : {?:}", discr_names);
/// println!("String table : {?:}", table);
/// ```
fn construct_string_table(analysis_table: &AnalysisTable) -> (Vec<Vec<String>>, Vec<String>) {

    // Map discriminants to an index to keep correct order in the table rows
    let mut discriminant_ids: HashMap<Discriminant<Token>, usize> = HashMap::new();

    // Map discriminants to their names
    let mut discriminant_names: Vec<String> = Vec::new();

    for (d, token) in analysis_table.discriminant_tokens.iter() {
        if !discriminant_ids.contains_key(d) {
            discriminant_ids.insert(d.clone(), discriminant_names.len());
            discriminant_names.push(generic_token_repr!(token));
        }
    }

    let nb_discriminants = discriminant_names.len();

    let nb_non_terminals = analysis_table.table.len();
    let mut str_table: Vec<Vec<String>> = vec![vec![String::new(); nb_discriminants]; nb_non_terminals];

    for i in 0 .. nb_non_terminals {
        for (discr, word) in &analysis_table.table[i] {

            let discr_id = discriminant_ids[discr];

            // Compute word string
            let word_str: String = word.iter()
                .map(|lexem| {
                    match lexem {
                        Lexem::NonTerminal(id) => analysis_table.non_terminal_names[*id].clone(),
                        Lexem::Terminal(token) => generic_token_repr!(token),
                    }
                })
                .collect();
            str_table[i][discr_id] = word_str;
        }
    }

    return (str_table, discriminant_names);
}
