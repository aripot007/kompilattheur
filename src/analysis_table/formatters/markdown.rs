use std::fmt;
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

                let (str_table, discriminant_names) = construct_string_table(&self.0);

                let nb_discriminants = discriminant_names.len();
                let nb_non_terminals = self.0.table.len();

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