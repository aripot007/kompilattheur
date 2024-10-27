use std::{collections::HashMap, fmt, mem::Discriminant};
use crate::common::types::token::Token;
use crate::parser::lexem::Lexem;

use super::super::analysis_table::AnalysisTable;
use super::generic_token_repr;


impl AnalysisTable {
    
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
                for (_, name) in discriminant_names.iter().enumerate() {
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