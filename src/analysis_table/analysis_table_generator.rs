use std::{fs::{self, read_to_string, File}, io::{self, BufRead, BufReader, Read}, iter::Product};

use crate::{analysis_table::{analysis_table::AnalysisTable, grammar::ParsedLexem}, parser::lexem::Lexem};

use super::grammar::{self, Grammar};

/// Parse the first lexem name of a string
fn parse_lexem_name(name: &str) -> &str{

    let quote_indices: Vec<_> = name.match_indices("\"").collect();

    if quote_indices.len() < 2 {
        panic!("Could not parse lexem name '{}'", name);
    }

    match name.get(quote_indices[0].0 + 1 .. quote_indices[1].0) {
        Some(name) => return name,
        None => panic!("Could not parse lexem name '{}'", name),
    };
}

/// Parse all lexem names in a string
fn parse_all_lexem_names(names_str: &str) -> Vec<String> {

    let mut names: Vec<String> = Vec::new();

    let mut quote_open = false;
    let mut current_lexem = String::new();

    for c in names_str.chars() {

        if c == '"' {
            if quote_open {
                // Finished reading lexem name
                names.push(current_lexem.clone());
                current_lexem = String::new();
                quote_open = false;
            } else {
                quote_open = true;
            }

        } else if quote_open {
            current_lexem.push(c);
        }

    }

    if quote_open {
        panic!("Unterminated quote in lexem names : {}", names_str)
    }

    names
}

/// Parse une grammaire
fn parse_grammar(input_file: &str) -> Grammar {

    let mut grammar = Grammar::new();

    let f = match File::open(input_file) {
        Ok(f) => f,
        Err(e) => panic!("Erreur lors de l'ouverture du fichier {} : {}", input_file, e),
    };

    let reader = fs::read_to_string(input_file).unwrap();

    let file_without_comments: String = reader
        .lines()
        .map(String::from)
        .filter(|l| !l.starts_with("#"))  // Remove comments
        .collect();

    let mut i = 0;
    for ruleset in file_without_comments.split(";") {

        if ruleset.len() == 0 {
            println!("Skipping empty ruleset {}", i);
            continue;
        }

        // Find rule separator
        let sep_indices: Vec<_> = ruleset.match_indices(":").collect();

        if sep_indices.len() == 0 {
            panic!("Could not find starting non terminal for rule {} : '{}'", i, ruleset);
        }

        let (start, products) = ruleset.split_at(sep_indices[0].0);

        // Get non-terminal
        let start = parse_lexem_name(start);

        // Remove ':' separator thats left in products
        let mut products = products.to_string();
        products.remove(0);

        let mut rule_nb = 0;
        for prod in products.split("|") {

            let produced_lexems = parse_all_lexem_names(prod);

            grammar.create_rule(start, produced_lexems);

            rule_nb += 1;
        }

        i += 1;
    }

    return grammar;
}

/// Génère une table d'analyse pour la grammaire contenue dans le fichier d'entrée
pub fn generate_analysis_table(input_file: &str, output_file: &str) -> AnalysisTable{
    
    let mut grammar = parse_grammar(input_file);

    println!("Non terminaux : ");

    for (name, lexem) in grammar.non_terminal_lexems.iter() {
        let Lexem::NonTerminal(id) = lexem.lexem else {
            panic!();
        };
        println!("{} ({})", name, id);
    }

    println!("Producteurs de mot vide : ");

    let producers = grammar.empty_word_producers();

    for p in producers {
        print!("{} ", p);
    }
    println!("");

    println!("Premiers : ");

    let firsts = grammar.firsts();

    let mut i = 0;
    for f in firsts {
        println!("P({}) : {:?}", i, f);
        i += 1;
    }

    println!("Suivants : ");
    
    let follows = grammar.follows();

    let mut i = 0;
    for f in follows {
        println!("S({}) : {:?}", i, f);
        i += 1;
    }

    return AnalysisTable::from(&grammar);

}