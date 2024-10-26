use crate::common::types::token::Token;
use crate::parser::lexem::Lexem;
use std::collections::HashMap;
use std::mem::{discriminant, Discriminant};

use super::grammar::{Grammar, ParsedLexem};

/// Représente une table d'analyse pour une grammaire
pub struct AnalysisTable {
    /// La table d'analyse. Pour chaque non terminal, contient une HashMap des tokens lu et de la production de la règle correspondante
    pub table: Vec<HashMap<Discriminant<Token>, Vec<Lexem>>>,
    pub non_terminal_names: Vec<String>,
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
