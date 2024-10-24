mod reader;
mod common;
mod lexer;
mod parser;
use lexer::lexer::Lexer;
use parser::generate_tree::generate_tree;
use clap::Parser;

/// MiniPython Compiler
#[derive(Parser)]
#[command(version)]
struct Args {

    /// Génère une table d'analyse à partir du fichier d'entrée
    #[arg(long="generate-parsing-table", action)]
    generate_parsing_table: bool,

    /// Le fichier à compiler
    #[arg(default_value="test_programs/hello_world.smolpp")]
    file: String,

    /// Fichier de sortie
    #[arg(short, long="output")]
    output_file: Option<String>,

}

fn main() {

    let args = Args::parse();

    let file_path = args.file;
    let lexer = Lexer::new(reader::new(file_path.clone()));

    for token in lexer {
        print!("{} ", token);
    }

    print!("\n");
    let lexer = Lexer::new(reader::new(file_path));
    generate_tree(lexer);
}

