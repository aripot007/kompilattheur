mod analysis_table_generator;
mod grammar;
mod generated_table;
mod analysis_table;
mod formatters;

use std::path::Path;
use std::sync::OnceLock;
pub use analysis_table_generator::generate_analysis_table;
pub use analysis_table::AnalysisTable;

static ANALYSIS_TABLE: OnceLock<AnalysisTable> = OnceLock::new();

/// Renvoie la table d'analyse globale du compilateur.
/// Panique si elle n'a pas été initialisée avec `setup_grammar`
pub fn get_analysis_table() -> &'static AnalysisTable {
    match ANALYSIS_TABLE.get() {
        Some(table) => table,
        None => panic!("Trying to use global analysis table without initializing it first"),
    }
}

/// Configure la grammaire utilisée par le compilateur. Si un fichier est spécifié, construit la table d'analyse en utilisant
/// cette grammaire, sinon, utilise celle intégrée au compilateur.
/// Cette fonction ne doit être appelée qu'une seule fois
/// 
/// Renvoie la table d'analyse configurée
pub fn setup_analysis_table(grammar_file: Option<&Path>) -> &'static AnalysisTable {

    let table = match grammar_file {
        Some(file) => generate_analysis_table(&file),
        None => generated_table::get_analysis_table(),
    };

    match ANALYSIS_TABLE.set(table) {
        Ok(_) => (),
        Err(_) => panic!("Error setting up global analysis table"),
    }

    return get_analysis_table();
}