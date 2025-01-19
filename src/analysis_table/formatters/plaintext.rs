use std::fmt;
use crate::analysis_table::{formatters::construct_string_table, NonTerminal};

use super::super::analysis_table::AnalysisTable;

impl AnalysisTable {
    /// Formate la table d'analyse dans un format lisible en texte clair
    pub fn display_plain<'a>(&'a self) -> impl fmt::Display + 'a {

        struct PlainAnalysisTable<'a>(&'a AnalysisTable);
        impl<'a> fmt::Display for PlainAnalysisTable<'a> {

            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

                let (str_table, term_discr_names, nonterm_list) = construct_string_table(&self.0);
                
                let nonterm_names: Vec<String> = nonterm_list.iter().map(NonTerminal::to_string).collect();

                let nb_discriminants = term_discr_names.len();

                // Initialize column size with non terminal name
                let mut column_sizes: Vec<usize> = term_discr_names.iter()
                                                    .map(String::len)
                                                    .collect();
        
                // Compute column sizes
                let nb_non_terminals = nonterm_names.len();
        
                for row in &str_table {
                    for (col, word) in row.iter().enumerate() {
        
                        if word.len() > column_sizes[col] {
                            column_sizes[col] = word.len();
                        }
                    }
                }
        
                // Add 2 spaces for the padding
                column_sizes = column_sizes.iter().map(|s| s + 2).collect();
        
                // Starting non-terminal column size
                let left_lexem_max_size = nonterm_names
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
                for (i, name) in term_discr_names.iter().enumerate() {
                    write!(f, "{:^width$}+", name, width=column_sizes[i])?;
                }
                write!(f, "\n")?;
        
                write!(f, "{}", sep_str)?;
        
                // Print table
                for i in 0..nb_non_terminals {
        
                    write!(f, "|{:^width$}|", nonterm_names[i], width=left_col_size)?;
        
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
