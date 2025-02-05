use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use clap_complete::Shell;

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
    PrintAnalysisTable(PrintTableArgs),

    /// Génère le fichier d'autocomplétion
    GenerateAutocompletion {
        #[arg()]
        shell: Shell,
    },
}

#[derive(Debug, Args)]
pub struct GenerateTableArgs {
    /// Le fichier contenant la grammmaire
    #[arg()]
    pub grammar_file: PathBuf,

    /// Le fichier de sortie
    #[arg(name = "output", short, long, default_value = "generated_table.rs")]
    pub output_file: PathBuf,

    /// Affiche également la table générée
    #[arg(short, long, action)]
    pub print_table: bool,

    /// Change le format d'affichage de la table
    #[arg(long, short, require_equals = true, num_args = 0..=1, value_enum, default_value_t=TableFormat::Plaintext, requires("print_table"))]
    pub format: TableFormat,

    /// Ajoute des commentaires
    #[arg(long, action, alias("comments"))]
    pub with_comments: bool,
}

#[derive(Debug, Args)]
#[command(flatten_help = true)]
pub struct PrintTableArgs {
    /// Utilise la grammaire contenue dans ce fichier à la place de celle du compilateur
    #[arg(long, short)]
    pub grammar_file: Option<PathBuf>,

    /// Un fichier dans lequel écrire la table, au lieu de la sortie standard
    #[arg(name = "output", short, long)]
    pub output_file: Option<PathBuf>,

    #[arg(long, short, num_args = 0..=1, value_enum, default_value_t=TableFormat::Plaintext)]
    pub format: TableFormat,

    /// Ajoute des commentaires
    #[arg(long, action, alias("comments"))]
    pub with_comments: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TableFormat {
    #[value(alias("plain"))]
    Plaintext,

    #[value(alias("md"))]
    Markdown,

    #[value(alias("rs"))]
    Rust,
}

#[derive(Debug, Args)]
pub struct CompileArgs {
    /// Utilise une autre grammaire que celle inclue dans le compilateur
    #[arg(long, short = 'g')]
    pub alternative_grammar: Option<PathBuf>,

    /// Le fichier à compiler
    #[arg()]
    pub file: Option<PathBuf>,

    /// Le fichier de sortie
    #[arg(name = "output", short, long, default_value = "p.out")]
    pub output_file: PathBuf,

    /// Étape de compilation à laquelle le compilateur s'arrête
    #[arg(short = 's', long, num_args = 0..=1, value_enum, default_value_t=TargetStep::AbstractTree, alias("step"))]
    pub target_step: TargetStep,

    /// Language de destination
    #[arg(long, short, num_args = 0..=1, value_enum, default_value_t=TargetLanguage::Html)]
    pub target: TargetLanguage,

    /// Lance le programme compilé (ou ouvre le fichier résultant en fonction de l'étape de compilation)
    #[arg(long, short, action)]
    pub run: bool,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetLanguage {
    /// Arbre mermaid
    #[value(alias("mmd"))]
    Mermaid,

    #[value()]
    Html,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetStep {
    /// Analyse lexicale seulement
    #[value(alias("lexer"), alias("tokens"))]
    Lexing,

    /// Arbre Syntaxique Concret
    #[value(alias("st"), alias("cst"))]
    ConcreteTree,

    /// Arbre Syntaxique Abstrait
    #[value(alias("ast"), alias("parsing"), alias("parser"))]
    AbstractTree,
}
