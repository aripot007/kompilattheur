mod reader;
mod common;
mod lexer;
mod parser;
mod analysis_table;
mod cli;
use std::{io::{stdout, Write}, path::PathBuf};
use analysis_table::{analysis_table::AnalysisTable, analysis_table_generator};
use clap::Parser;
use cli::{Commands, CompileArgs, GenerateTableArgs, PrintTableArgs};
use lexer::lexer::Lexer;
use parser::generate_tree::generate_tree;
use std::fs::File;

fn main() {

    let args = cli::Cli::parse();

    dbg!(&args);

    match args.command {
        Some(Commands::GenerateAnalysisTable(generate_args)) => generate_analysis_table(generate_args),
        Some(Commands::PrintAnalysisTable(print_args)) => print_analysis_table(print_args),
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

    let table = analysis_table_generator::generate_analysis_table(&args.grammar_file);

    let mut file = match File::create(&args.output_file) {
        Ok(f) => f,
        Err(e) => panic!("Error opening file {:?} : {}", &args.output_file, e),
    };
    
    write!(file, "{}", table).expect("Error writing to file");

    if args.print_table {
        _print_analysis_table(&table, None, args.format);
    }
    
}

fn print_analysis_table(args: PrintTableArgs) {
    let table = match args.grammar_file {
        Some(file) => analysis_table_generator::generate_analysis_table(&file),
        None => analysis_table::generated_table::get_analysis_table(),
    };

   _print_analysis_table(&table, args.output_file, args.format);
}

fn _print_analysis_table(table: &AnalysisTable, output_file: Option<PathBuf>, format: cli::TableFormat) {
    let mut out_handle: Box<dyn std::io::Write> = match output_file {
        Some(file) => Box::new(File::create(file).expect("Error opening output file")),
        None => Box::new(stdout()),
    };

    match format {
        cli::TableFormat::Plaintext => write!(out_handle, "{}", table.display_plain()),
        cli::TableFormat::Markdown => write!(out_handle, "{}", table.display_markdown()),
    }.expect("Error while printing table");
}