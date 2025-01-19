use crate::common::types::Token;
use crate::parser::Lexem;
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::mem::{discriminant, Discriminant};
use super::generated_table::NonTerminal;
use super::grammar::{Grammar, ParsedLexem};

/// Représente une table d'analyse pour une grammaire
pub struct AnalysisTable {
    /// La table d'analyse. Pour chaque non terminal, contient une HashMap des tokens lu et de la production de la règle correspondante
    pub table: HashMap<NonTerminal, HashMap<Discriminant<Token>, Vec<Lexem>>>,

    /// Permet de map un discriminant à un token pour le debug
    pub(super) tokens_discriminants: HashMap<Discriminant<Token>, Token>,

    pub(super) used_non_terminals: HashSet<NonTerminal>,
}

impl AnalysisTable {

    pub fn get(&self, non_terminal: &NonTerminal, token: &Token) -> Option<&Vec<Lexem>> {
        self.table.get(non_terminal)?.get(&discriminant(token))
    }

    pub fn get_expected_tokens(&self, non_terminal: &NonTerminal) -> Vec<&Token> {
        if let Some(map) = self.table.get(non_terminal) {
            map.keys().map(|d| self.tokens_discriminants.get(d).unwrap()).collect()
        } else {
            Vec::new()
        }
    }

    /// Essaie d'ajouter une règle une case de la table d'analyse.
    /// Si une règle est déjà présente (grammaire non LL(1)), panique
    fn try_set(&mut self, left_non_terminal: &ParsedLexem, token: &Token, production: &Vec<ParsedLexem>) {
        
        let non_terminal = match &left_non_terminal.lexem {
            Lexem::Terminal(t) => panic!("Trying to fill analysis table with the terminal {} on the left of a rule", t),
            Lexem::NonTerminal(nt) => nt,
        };

        // Convert ParsedLexem to Lexem
        let production_vec: Vec<Lexem> = production.iter().map(|l| l.lexem.clone()).collect();

        if !self.table.contains_key(&non_terminal) {
            self.table.insert(non_terminal.clone(), HashMap::new());
        }

        let non_term_line = self.table.get_mut(non_terminal).unwrap();

        match non_term_line.insert(discriminant(token), production_vec) {
            Some(prod) => {
                let p2: Vec<Lexem> = production.iter().map(|l| l.lexem.clone()).collect();
                panic!("Grammar is not LL(1) : conflicting rules on stack=[{left_non_terminal}] read=[{token}] : {prod:?} / {p2:?}")
            },
            None => (),
        };

        // Save the token associated to the discriminant
        self.tokens_discriminants.insert(discriminant(token), token.clone());
    }
}

impl From<&Grammar> for AnalysisTable {

    fn from(grammar: &Grammar) -> Self {
        
        // Initialize an empty analysis table

        let analysis_table: HashMap<NonTerminal, HashMap<Discriminant<Token>, Vec<Lexem>>> = HashMap::new();

        let mut table = AnalysisTable {
            table: analysis_table,
            tokens_discriminants: HashMap::new(),
            used_non_terminals: grammar.non_terminal_lexems
                .iter()
                .filter_map(|pl| match &pl.lexem {
                    Lexem::Terminal(_) => None,
                    Lexem::NonTerminal(nt) => Some(nt.clone()),
                })
                .collect()
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

impl Display for AnalysisTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display_plain().fmt(f)
    }
}
