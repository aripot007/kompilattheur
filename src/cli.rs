use clap::Parser;

/// MiniPython Compiler
#[derive(Parser)]
#[command(version)]
pub struct Args {

    /// Génère une table d'analyse à partir du fichier d'entrée
    #[arg(long="generate-analysis-table", action)]
    pub generate_alanysis_table: bool,

    /// Le fichier à compiler
    #[arg(default_value="test_programs/hello_world.smolpp")]
    pub file: String,

    /// Fichier de sortie
    #[arg(short, long="output")]
    pub output_file: Option<String>,

}