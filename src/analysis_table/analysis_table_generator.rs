use std::{fs, path::Path};

use crate::analysis_table::AnalysisTable;

use super::grammar::Grammar;

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
fn parse_grammar(input_file: &Path) -> Grammar {

    let mut grammar = Grammar::new();

    let reader = match fs::read_to_string(input_file) {
        Ok(s) => s,
        Err(e) =>  panic!("Erreur lors de l'ouverture du fichier {:?} : {}", input_file, e),
    };

    let file_without_comments: String = reader
        .lines()
        .map(String::from)
        .filter(|l| !l.starts_with("#"))  // Remove comments
        .collect();

    let mut i = 0;
    for ruleset in file_without_comments.split(";") {

        if ruleset.len() == 0 {
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

        for prod in products.split("|") {

            let produced_lexems = parse_all_lexem_names(prod);

            grammar.create_rule(start, produced_lexems);

        }

        i += 1;
    }

    // Precompute empty word producers, firsts and follows
    grammar.empty_word_producers();
    grammar.firsts();
    grammar.follows();

    return grammar;
}

/// Génère une table d'analyse pour la grammaire contenue dans le fichier d'entrée
pub fn generate_analysis_table(input_file: &Path) -> AnalysisTable{
    
    let grammar = parse_grammar(input_file);
    return AnalysisTable::from(&grammar);

}


#[cfg(test)]
mod tests {

    use std::path::PathBuf;
    use crate::{analysis_table::grammar::{ParsedLexem, Rule}, common::types::token::Token};

    use super::{parse_all_lexem_names, parse_grammar, parse_lexem_name};

    #[test]
    fn test_parse_lexem_name() {
        let s = "garbage \"thename\" garbage";
        assert_eq!("thename", parse_lexem_name(s));
    }
    
    #[test]
    fn test_parse_lexem_name_multiple_quotes() {
        let s = "garbage \"thename\" not \" the \"\"name";
        assert_eq!("thename", parse_lexem_name(s));
    }

    #[test]
    #[should_panic]
    fn test_parse_lexem_name_invalid_quotes() {
        let s = "unterminated \"quote";
        println!("Parsed name : '{}'", parse_lexem_name(s)) ;
    }

    #[test]
    #[should_panic]
    fn test_parse_lexem_name_no_quotes() {
        let s = "no quote here";
        println!("Parsed name : '{}'", parse_lexem_name(s)) ;
    }

    #[test]
    fn test_parse_all_lexem_names() {

        // []
        let s = "the name is a lie";
        let lexems = parse_all_lexem_names(s);
        assert_eq!(0, lexems.len());

        // ["a single name"]
        let s = "\"a single name\"";
        let lexems = parse_all_lexem_names(s);
        assert_eq!(1, lexems.len());
        assert_eq!("a single name", lexems[0]);

        // ["lots", "of", "names"]
        let s = "nothing \"lots\" garbage \"of\"\"names\"";
        let lexems = parse_all_lexem_names(s);
        assert_eq!(3, lexems.len());
        assert_eq!(vec![String::from("lots"), String::from("of"), String::from("names")], lexems);

    }

    #[test]
    #[should_panic]
    fn test_parse_all_lexem_names_panic() {

        let s = "\"names\" with \"unterminated quotes";
        let lexems = parse_all_lexem_names(s);
        println!("Parsed lexems : {:?}", lexems)

    }

    #[test]
    fn test_parse_grammar() {

        let mut grammar_file = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        grammar_file.push("resources/test/test_grammar.txt");

        /*
        Grammar : 

        A -> + | B <integer>
        B -> A <for> | ^
        
        Empty word producers = {B}

        P(A) = {+, <integer>}
        P(B) = {+, <integer>}

        S(A) = {EOF, <for>}
        S(B) = {<integer>}

        */

        let mut g = parse_grammar(&grammar_file);

        assert_eq!(4, g.rules.len());

        // Force set rule id to 0 for easier comparison
        g.rules.iter_mut().for_each(|r| r.id = 0);

        let a: ParsedLexem = g.non_terminal_lexems["A"].clone();
        let b: ParsedLexem = g.non_terminal_lexems["B"].clone();

        let add = g.get_lexem("+");
        let int = g.get_lexem("<integer>");
        let for_loop = g.get_lexem("for");

        let rules: Vec<Rule> = vec![

            // A -> +
            Rule {id: 0, start: a.clone(), production: vec![add.clone()]},

            // A -> B <integer>
            Rule {id: 0, start: a.clone(), production: vec![b.clone(), int.clone()]},

            // B -> A <for>
            Rule {id: 0, start: b.clone(), production: vec![a.clone(), for_loop.clone()]},

            // B -> ^
            Rule {id: 0, start: b.clone(), production: vec![]},
        ];

        // Check rules
        for r in &g.rules {
            assert!(rules.contains(r));
        }

        // Check empty word producers
        let v = &g.empty_word_producers();
        assert_eq!(1, v.len());
        assert!(g.produces_empty_word_unmut(&b.lexem));


        // Check firsts
        // P(A) = {+, <integer>}
        // P(B) = {+, <integer>}
        let a_firsts = &g.get_firsts_unmut(&a.lexem);
        assert!(a_firsts.contains(&Token::Add));
        assert!(a_firsts.contains(&Token::integer(0)));
        assert_eq!(2, a_firsts.len());

        let b_firsts = &g.get_firsts_unmut(&b.lexem);
        assert!(b_firsts.contains(&Token::Add));
        assert!(b_firsts.contains(&Token::integer(0)));
        assert_eq!(2, b_firsts.len());


        // Check follows
        // S(A) = {EOF, <for>}
        // S(B) = {<integer>}
        let a_follows = &g.get_follows_unmut(&a);
        assert_eq!(2, a_follows.len());
        assert!(a_follows.contains(&Token::EOF));
        assert!(a_follows.contains(&Token::For));

        let b_follows = &g.get_follows_unmut(&b);
        assert_eq!(1, b_follows.len());
        assert!(b_follows.contains(&Token::integer(0)));
        
    }

}