mod analysis_table;
mod ast;
mod cli;
mod common;
mod lexer;
mod parser;
mod reader;
use analysis_table::{get_analysis_table, setup_analysis_table, AnalysisTable};
use clap::{CommandFactory, Parser};
use cli::{Commands, CompileArgs, GenerateTableArgs, PrintTableArgs};
use common::types::Tree;
use ast::generate_ast;
use lexer::Lexer;
use parser::{generate_tree, Lexem};
use std::fs::File;
use std::io::{self, stdout, Write};
use std::sync::OnceLock;
use webbrowser;

static FILE_PATH: OnceLock<String> = OnceLock::new();

fn main() {
    let args = cli::Cli::parse();

    match args.command {
        Some(Commands::GenerateAnalysisTable(generate_args)) => {
            generate_analysis_table(generate_args)
        }
        Some(Commands::PrintAnalysisTable(print_args)) => print_analysis_table(print_args),
        Some(Commands::GenerateAutocompletion { shell }) => {
            let mut cmd = cli::Cli::command();
            let name = cmd.get_name().to_string();
            clap_complete::generate(shell, &mut cmd, name, &mut io::stdout());
            return;
        }
        None => compile(args.compile),
    }
}

fn compile(args: CompileArgs) {
    let Some(file_path) = args.file else {
        eprintln!("Input file required");
        return;
    };

    FILE_PATH
        .set(file_path.to_str().unwrap().to_string())
        .unwrap();

    let lexer = Lexer::new(reader::new(&file_path));

    setup_analysis_table(args.alternative_grammar.as_deref());

    let table = get_analysis_table();
    let (tree, accept, error): (Tree<Lexem>, bool, bool) =
        generate_tree(lexer, &table);
    println!("Accepted: {}, Error: {}", accept, error);
    if args.syntax_tree {
        let mut output_file = File::create(&args.output_file).expect("Error opening output file");
        write!(output_file, "{}", tree.borrow().generate_html()).expect("error writing to output");
        if let Some(output_path_str) = &args.output_file.to_str() {
            if webbrowser::open(output_path_str).is_err() {
                eprintln!("Failed to open the HTML file in the web browser.");
            }
        } else {
            eprintln!("Failed to convert output path to string.");
        }
        return;
    }

    generate_ast(tree.clone());

    let mut output_file = File::create(&args.output_file).expect("Error opening output file");
    write!(output_file, "{}", tree.borrow().generate_html()).expect("error writing to output");

    if let Some(output_path_str) = &args.output_file.to_str() {
        if webbrowser::open(output_path_str).is_err() {
            eprintln!("Failed to open the HTML file in the web browser.");
        }
    } else {
        eprintln!("Failed to convert output path to string.");
    }
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
            with_comments: args.with_comments,
        };
        _print_analysis_table(&table, print_args);
    }
}

fn print_analysis_table(args: PrintTableArgs) {
    setup_analysis_table(args.grammar_file.as_deref());

    _print_analysis_table(&get_analysis_table(), args);
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
    }
    .expect("Error while printing table");
}
