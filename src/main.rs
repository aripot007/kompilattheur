mod reader;
mod common;
mod lexer;
mod parser;
mod analysis_table;
mod cli;
use std::{io::Write, path::PathBuf};
use analysis_table::analysis_table_generator;
use clap::Parser;
use cli::{Commands, CompileArgs, GenerateTableArgs};
use lexer::lexer::Lexer;
use parser::generate_tree::generate_tree;
use std::fs::File;

fn main() {

    let args = cli::Cli::parse();

    dbg!(&args);

    match args.command {
        Some(Commands::GenerateAnalysisTable(generate_args)) => generate_analysis_table(generate_args),
        Some(Commands::PrintAnalysisTable { grammar_file }) => print_analysis_table(grammar_file),
        None => compile(args.compile),
    }

}

fn compile(args: CompileArgs) {

    let Some(file_path) = args.file else {
        eprintln!("Input file required");
        return;
    };

    let lexer = Lexer::new(reader::new(&file_path));
    for token in lexer {
        print!("{} ", token);
    }

    print!("\n");
    let lexer = Lexer::new(reader::new(&file_path));
    generate_tree(lexer);
}

fn generate_analysis_table(args: GenerateTableArgs) {

    let grammar = analysis_table_generator::generate_analysis_table(&args.grammar_file);

    if args.print_table {
        println!("{}", grammar);
    }

    let mut file = match File::create(&args.output_file) {
        Ok(f) => f,
        Err(e) => panic!("Error opening file {:?} : {}", &args.output_file, e),
    };
    
    write!(file, "{}", grammar).expect("Error writing to file");
    
}

fn print_analysis_table(grammar_file: Option<PathBuf>) {
    let grammar = match grammar_file {
        Some(file) => analysis_table_generator::generate_analysis_table(&file),
        None => analysis_table::generated_table::get_analysis_table(),
    };

    println!("{}", grammar);
}
