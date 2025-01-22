mod analysis_table;
mod ast;
mod cli;
mod common;
mod lexer;
mod parser;
mod reader;
use analysis_table::{get_analysis_table, setup_analysis_table, AnalysisTable};
use clap::{CommandFactory, Parser};
use cli::{Commands, CompileArgs, GenerateTableArgs, PrintTableArgs, TargetStep};
use common::symbol_table::{enter_scope, exit_scope, get_scope, init_symbol_table, Symbol};
use common::types::{FileElement, Tree};
use ast::generate_ast;
use lexer::Lexer;
use parser::{generate_tree, Lexem};
use std::fs::File;
use std::io::{self, stdout, Write};
use std::process::exit;
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
        },
        Some(Commands::SymbolTableExample) => symbol_table_example(),
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

    let mut output_file = File::create(&args.output_file).expect("Error opening output file");

    let lexer = Lexer::new(reader::new(&file_path));

    if args.target_step == TargetStep::Lexing {
        for token in lexer {
            write!(output_file, "{} ", token.element).expect("error writing to output");
        }
        write!(output_file, "\n").expect("error writing to output");
        return;
    }

    setup_analysis_table(args.alternative_grammar.as_deref());

    let table = get_analysis_table();
    let (tree, accept, error): (Tree<FileElement<Lexem>>, bool, bool) =
        generate_tree(lexer, &table);

    // println!("Accepted: {}, Error: {}", accept, error);

    if args.target_step == TargetStep::ConcreteTree {
        write!(output_file, "{}", tree.borrow().generate_html()).expect("error writing to output");
        if let Some(output_path_str) = &args.output_file.to_str() {
            if args.run && webbrowser::open(output_path_str).is_err() {
                eprintln!("Failed to open the HTML file in the web browser.");
            }
        } else {
            eprintln!("Failed to convert output path to string.");
        }
        return;
    }

    if error || !accept {
        eprintln!("Parsing ended with errors. Aborting");
        exit(1);
    }

    let ast = generate_ast(tree.clone());

    let display_ast: Tree<String> = ast.into();

    let mut output_file = File::create(&args.output_file).expect("Error opening output file");
    write!(output_file, "{}", display_ast.borrow().generate_html()).expect("error writing to output");

    if let Some(output_path_str) = &args.output_file.to_str() {
        if args.run && webbrowser::open(output_path_str).is_err() {
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

fn symbol_table_example() {
    let (node, root) = init_symbol_table();
    node.borrow_mut().insert_symbol(1, (Symbol::Function(),));
    node.borrow_mut().insert_symbol(2, (Symbol::Variable(),));

    let node = enter_scope(node);
    node.borrow_mut().insert_symbol(3, (Symbol::Parameter(),));
    node.borrow_mut().insert_symbol(4, (Symbol::Variable(),));
    node.borrow_mut().insert_symbol(5, (Symbol::Function(),));

    let node = enter_scope(node);
    node.borrow_mut().insert_symbol(6, (Symbol::Function(),));
    node.borrow_mut().insert_symbol(7, (Symbol::Variable(),));

    let node = exit_scope(node);
    node.borrow_mut().insert_symbol(8, (Symbol::Function(),));

    let node = enter_scope(node);

    let node = get_scope(node, 0).unwrap();
    node.borrow_mut().insert_symbol(9, (Symbol::Function(),));

    let node = enter_scope(node);
    node.borrow_mut().insert_symbol(10, (Symbol::Variable(),));

    let node = exit_scope(node);

    let node = enter_scope(node);
    let node = exit_scope(node);

    assert_eq!(
        node.borrow().get_value().index,
        root.borrow().get_value().index
    );

    let res = root.borrow().generate_unsafe_mermaid();
    let mut output_file = File::create("p.out").expect("Error opening output file");
    writeln!(output_file, "{}", res).expect("Error writing to output file");
    println!("{}", res)

}
