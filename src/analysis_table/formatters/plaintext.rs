use std::fmt;
use crate::analysis_table::formatters::construct_string_table;

use super::super::analysis_table::AnalysisTable;

impl AnalysisTable {
    /// Formate la table d'analyse dans un format lisible en texte clair
    pub fn display_plain<'a>(&'a self) -> impl fmt::Display + 'a {

        struct PlainAnalysisTable<'a>(&'a AnalysisTable);
        impl<'a> fmt::Display for PlainAnalysisTable<'a> {

            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {

                let (str_table, discriminant_names) = construct_string_table(&self.0);
                
                let nb_discriminants = discriminant_names.len();

                // Initialize column size with non terminal name
                let mut column_sizes: Vec<usize> = discriminant_names.iter()
                                                    .map(String::len)
                                                    .collect();
        
                // Compute column sizes
                let nb_non_terminals = self.0.table.len();
        
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
