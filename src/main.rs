mod reader;
mod common;
mod lexer;
mod parser;
mod analysis_table;
mod cli;
use std::io::{stdout, Write};
use analysis_table::AnalysisTable;
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

    let table = match &args.alternative_grammar {
        Some(file) => analysis_table::generate_analysis_table(&file),
        None => analysis_table::get_analysis_table(),
    };

    generate_tree(lexer, &table);
}

fn generate_analysis_table(args: GenerateTableArgs) {

    let table = analysis_table::generate_analysis_table(&args.grammar_file);

    let mut file = match File::create(&args.output_file) {
        Ok(f) => f,
        Err(e) => panic!("Error opening file {:?} : {}", &args.output_file, e),
    };
    
    write!(file, "{}", table.display_rust(args.with_comments)).expect("Error writing to file");

    if args.print_table {
        let print_args = PrintTableArgs {
            output_file: None,
            grammar_file: None,
            format: args.format,
            with_comments: args.with_comments
        };
        _print_analysis_table(&table, print_args);
    }
    
}

fn print_analysis_table(args: PrintTableArgs) {
    let table = match &args.grammar_file {
        Some(file) => analysis_table::generate_analysis_table(&file),
        None => analysis_table::get_analysis_table(),
    };

   _print_analysis_table(&table, args);
}

fn _print_analysis_table(table: &AnalysisTable, args: PrintTableArgs) {
    let mut out_handle: Box<dyn std::io::Write> = match args.output_file {
        Some(file) => Box::new(File::create(file).expect("Error opening output file")),
        None => Box::new(stdout()),
    };

    match args.format {
        cli::TableFormat::Plaintext => write!(out_handle, "{}", table.display_plain()),
        cli::TableFormat::Markdown => write!(out_handle, "{}", table.display_markdown()),
        cli::TableFormat::Rust => write!(out_handle, "{}", table.display_rust(false)),
    }.expect("Error while printing table");
}