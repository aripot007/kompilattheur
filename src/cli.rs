use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(args_conflicts_with_subcommands = true)]
#[command(flatten_help = true)]
pub struct Cli {

    #[command(subcommand)]
    command: Option<Commands>,

    /// Utilise une autre grammaire que celle inclue dans le compilateur
    #[arg(long, short='g')]
    alternative_grammar: Option<PathBuf>,

    #[command(flatten)]
    compile: CompileArgs,
}

#[derive(Debug, Subcommand, Clone)]
enum Commands {

    /// Génère la table d'analyse de la grammaire passée en paramètres
    GenerateAnalysisTable {

        /// Le fichier contenant la grammmaire
        #[arg()]
        grammar_file: PathBuf,

        /// Le fichier de sortie
        #[arg(name="output", short, long, default_value="generated_table.rs")]
        output_file: Option<PathBuf>,

        /// Affiche également la table générée au format markdown
        #[arg(short, long, action)]
        print: bool,
    },

    /// Affiche la table d'analyse de la grammaire du compilateur au format markdown
    PrintAnalysisTable {

        /// Utilise la grammaire contenue dans ce fichier à la place de celle du compilateur
        #[arg(long, short)]
        grammar_file: Option<PathBuf>,
    },
}

#[derive(Debug, Args, Clone)]
#[command(flatten_help = true)]
struct CompileArgs {

    /// Le fichier à compiler
    #[arg()]
    file: Option<PathBuf>,

    /// Le fichier de sortie
    #[arg(name="output", short, long, default_value="p.out")]
    output_file: Option<PathBuf>,

}