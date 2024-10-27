mod reader;
mod common;
mod lexer;
mod parser;
mod cli;
mod analysis_table;
use analysis_table::analysis_table_generator::generate_analysis_table;
use lexer::lexer::Lexer;
use parser::generate_tree::generate_tree;
use clap::Parser;


fn main() {

    let args = cli::Args::parse();

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

