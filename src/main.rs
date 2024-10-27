mod reader;
mod common;
mod lexer;
mod parser;
mod analysis_table;
use analysis_table::analysis_table_generator::generate_analysis_table;
use lexer::lexer::Lexer;
use parser::generate_tree::generate_tree;
use clap::Parser;

/// MiniPython Compiler
#[derive(Parser)]
#[command(version)]
struct Args {

    /// Génère une table d'analyse à partir du fichier d'entrée
    #[arg(long="generate-analysis-table", action)]
    generate_alanysis_table: bool,

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

    if args.generate_alanysis_table {

        let output_file = match args.output_file {
            Some(path) => path,
            None => "analysis_table.rs".to_string(),
        };

        let table = generate_analysis_table(&file_path);
        println!("{}", table);
        return;
    }

    let lexer = Lexer::new(reader::new(file_path.clone()));
    for token in lexer {
        print!("{} ", token);
    }

    print!("\n");
    let lexer = Lexer::new(reader::new(file_path));
    generate_tree(lexer);
}

