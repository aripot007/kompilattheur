mod markdown;
mod plaintext;
mod rust;
use super::analysis_table::AnalysisTable;
use super::NonTerminal;
use crate::common::types::Token;
use crate::parser::Lexem;
use std::{collections::HashMap, mem::Discriminant};

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

pub(super) use generic_token_repr;

/// Construct a `Vec<Vec<String>>` with the text that should be in each cell of the table.
/// Also returns a `discriminants_names` Vec that maps a discriminant id to their display name.
///
/// ```text
/// let table, term_discr_names, nonterm_discr_names = construct_string_table(&analysis_table);
///
/// println!("Terminals : {?:}", discr_names);
/// println!("String table : {?:}", table);
/// ```
fn construct_string_table(
    analysis_table: &AnalysisTable,
) -> (Vec<Vec<String>>, Vec<String>, Vec<NonTerminal>) {
    // Map discriminants to an index to keep correct order in the table rows
    let mut term_discriminant_ids: HashMap<Discriminant<Token>, usize> = HashMap::new();

    // Map discriminants to their names
    let mut term_discriminant_names: Vec<String> = Vec::new();

    for (d, token) in analysis_table.tokens_discriminants.iter() {
        if !term_discriminant_ids.contains_key(d) {
            term_discriminant_ids.insert(d.clone(), term_discriminant_names.len());
            term_discriminant_names.push(generic_token_repr!(token));
        }
    }

    // Map NonTerminal to an index to keep correct order in the table rows
    let mut nonterm_ids: HashMap<NonTerminal, usize> = HashMap::new();

    // Map NonTerm id to the correspondint NonTerm
    let mut nonterm_list: Vec<NonTerminal> = Vec::new();

    for nt in &analysis_table.used_non_terminals {
        if !nonterm_ids.contains_key(&nt) {
            nonterm_ids.insert(nt.clone(), nonterm_list.len());
            nonterm_list.push(nt.clone());
        }
    }

    let nb_discriminants = term_discriminant_names.len();

    let nb_non_terminals = analysis_table.table.len();
    let mut str_table: Vec<Vec<String>> =
        vec![vec![String::new(); nb_discriminants]; nb_non_terminals];

    for (nt, line) in &analysis_table.table {
        if !nonterm_ids.contains_key(&nt) {
            nonterm_ids.insert(nt.clone(), nonterm_list.len());
            nonterm_list.push(nt.clone());
        }

        let i = nonterm_ids[nt];

        for (discr, word) in line {
            let discr_id = term_discriminant_ids[discr];

            // Compute word string
            let word_str: String = word
                .iter()
                .map(|lexem| match lexem {
                    Lexem::NonTerminal(nt) => nt.to_string(),
                    Lexem::Terminal(token) => generic_token_repr!(token),
                })
                .collect();
            str_table[i][discr_id] = word_str;
        }
    }

    return (str_table, term_discriminant_names, nonterm_list);
}
