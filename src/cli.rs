use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct Cli {

    #[command(subcommand)]
    pub command: Option<Commands>,

    #[command(flatten)]
    pub compile: CompileArgs,
}

#[derive(Debug, Subcommand)]
pub enum Commands {

    /// Génère la table d'analyse de la grammaire passée en paramètres
    GenerateAnalysisTable(GenerateTableArgs),

    /// Affiche la table d'analyse de la grammaire du compilateur au format markdown
    PrintAnalysisTable {

        /// Utilise la grammaire contenue dans ce fichier à la place de celle du compilateur
        #[arg(long, short)]
        grammar_file: Option<PathBuf>,
    },
}

#[derive(Debug, Args)]
pub struct GenerateTableArgs {

    /// Le fichier contenant la grammmaire
    #[arg()]
    pub grammar_file: PathBuf,

    /// Le fichier de sortie
    #[arg(name="output", short, long, default_value="generated_table.rs")]
    pub output_file: PathBuf,

    /// Affiche également la table générée au format markdown
    #[arg(short, long, action)]
    pub print_table: bool,
}

#[derive(Debug, Args)]
pub struct CompileArgs {

    /// Utilise une autre grammaire que celle inclue dans le compilateur
    #[arg(long, short='g')]
    pub alternative_grammar: Option<PathBuf>,
    
    /// Le fichier à compiler
    #[arg()]
    pub file: Option<PathBuf>,
    
    /// Le fichier de sortie
    #[arg(name="output", short, long, default_value="p.out")]
    pub output_file: PathBuf,
}