use std::fmt;
use crate::analysis_table::NonTerminal;

use super::construct_string_table;
use super::super::analysis_table::AnalysisTable;


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

                let (str_table, term_discr_names, nonterm_list) = construct_string_table(&self.0);

                let nonterm_names: Vec<String> = nonterm_list.iter().map(NonTerminal::to_string).collect();


                let nb_discriminants = term_discr_names.len();
                let nb_non_terminals = self.0.table.len();

                // Print header
                write!(f, "| |")?;
                for (_, name) in term_discr_names.iter().enumerate() {
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

                    write!(f, "|{}|", escape_markdown!(nonterm_names[i]))?;

                    for j in 0..nb_discriminants {
                        write!(f, "{}|", escape_markdown!(str_table[i][j]))?;
                    }

                    write!(f, "\n")?;
                }
                Ok(())
            }
        }
        MarkdownAnalysisTable(self)
    }
}