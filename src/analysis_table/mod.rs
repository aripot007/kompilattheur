mod analysis_table_generator;
mod grammar;
mod generated_table;
mod analysis_table;
mod formatters;

pub use analysis_table_generator::generate_analysis_table;
pub use analysis_table::AnalysisTable;

/// Renvoie la table d'analyse intégrée au compilateur
pub fn get_analysis_table() -> AnalysisTable {
    return generated_table::get_analysis_table();
}