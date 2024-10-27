use std::{collections::HashMap, fmt, mem::Discriminant};
use crate::common::types::token::Token;
use crate::parser::lexem::Lexem;

use super::super::analysis_table::AnalysisTable;
use super::generic_token_repr;

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
}
