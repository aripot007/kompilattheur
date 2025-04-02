mod analysis_table;
mod asm;
mod ast;
mod cli;
mod common;
mod lexer;
mod parser;
mod reader;
mod typing;
use analysis_table::{get_analysis_table, setup_analysis_table, AnalysisTable};
use asm::codegen::CodeGen;
use asm::execute::execute_binary;
use ast::generate_ast;
use clap::{CommandFactory, Parser};
use cli::{Commands, CompileArgs, GenerateTableArgs, PrintTableArgs, TargetStep};
use common::symbol_table::{
    enter_scope, exit_scope, get_scope, init_symbol_table, set_symbols_offset, Symbol, SymbolTableElement,
};
use common::types::{FileElement, Tree};
use inkwell::context::Context;
use inkwell::targets::FileType::{Assembly, Object};
use inkwell::targets::TargetMachine;
use inkwell::OptimizationLevel;
use lexer::Lexer;
use parser::{generate_tree, Lexem};
use std::fs::File;
use std::io::{self, stdout, Write};
use std::process::exit;
use std::sync::OnceLock;
use typing::{parse_types, Type};
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
        _ => compile(args.compile),
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

    if args.target_step == TargetStep::Lexing {
        let mut output_file = File::create(&args.output_file.unwrap_or("p.lex".into()))
            .expect("Error opening output file");

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
        let output_file_name;

        if let Some(name) = args.output_file {
            output_file_name = name;
        } else {
            output_file_name = match &args.target {
                cli::TargetLanguage::Mermaid => "p.mmd".into(),
                cli::TargetLanguage::Html => "p.html".into(),
                cli::TargetLanguage::Assembly
                | cli::TargetLanguage::Object
                | cli::TargetLanguage::Binary => {
                    eprintln!("Incompatible target language for concrete tree");
                    exit(1);
                }
            }
        }

        let mut output_file = File::create(&output_file_name).expect("Error opening output file");
        write!(output_file, "{}", tree.borrow().generate_html()).expect("error writing to output");
        if let Some(output_path_str) = output_file_name.to_str() {
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

    let ast: ast::nodes::Root = generate_ast(tree.clone());

    let (ast, symbol_table, context) = parse_types(ast);

    let symbol_table = set_symbols_offset(symbol_table);

    for warning in context.warnings {
        warning.display();
    }

    if !context.errors.is_empty() {
        for error in context.errors {
            error.display();
        }
        eprintln!("Typing ended with errors. Aborting");
        exit(1);
    }

    if args.show_symbol_table {
        let mut symbol_table_file =
            File::create("symbol_table.mmd").expect("Error opening symbol table file");
        write!(
            symbol_table_file,
            "{}",
            symbol_table.borrow().generate_unsafe_mermaid()
        )
        .expect("Error writing symbol table");
        println!("Symbol table written to symbol_table.mmd");
    }

    if args.target_step == TargetStep::AbstractTree {
        let display_ast: Tree<String> = ast.into();

        let output_file_name;

        if let Some(name) = args.output_file {
            output_file_name = name;
        } else {
            output_file_name = match &args.target {
                cli::TargetLanguage::Mermaid => "p.mmd".into(),
                cli::TargetLanguage::Html => "p.html".into(),
                cli::TargetLanguage::Assembly
                | cli::TargetLanguage::Object
                | cli::TargetLanguage::Binary => {
                    eprintln!("Incompatible target language for abstract tree");
                    exit(1);
                }
            }
        }

        let mut output_file = File::create(&output_file_name).expect("Error opening output file");

        write!(output_file, "{}", display_ast.borrow().generate_html())
            .expect("error writing to output");

        if let Some(output_path_str) = output_file_name.to_str() {
            if args.run && webbrowser::open(output_path_str).is_err() {
                eprintln!("Failed to open the HTML file in the web browser.");
            }
        } else {
            eprintln!("Failed to convert output path to string.");
        }
        return;
    }

    let context = Context::create();
    let target_triple = TargetMachine::get_default_triple();
    println!("Target triple: {}", target_triple.to_string());
    let mut codegen = CodeGen::create(&context, &target_triple, &file_path).unwrap();
    codegen.generate_llvm(&ast);
    if let Err(e) = codegen.verify() {
        eprintln!("LLVM codegen ended with errors: {}", e);
        exit(1);
    }

    if args.target_step == TargetStep::LLVMIR {
        let mut output_file = File::create(&args.output_file.unwrap_or("p.ll".into()))
            .expect("Error opening output file");
        let llvm_ir = codegen.module.to_string();
        write!(output_file, "{}", llvm_ir).expect("error writing to output");
        return;
    }

    if args.jit {
        let execution_engine = codegen
            .module
            .create_jit_execution_engine(OptimizationLevel::None)
            .map_err(|e| format!("Failed to create JIT execution engine: {}", e));
        match execution_engine {
            Ok(engine) => {
                asm::execute::execute(engine);
            }
            Err(e) => {
                eprintln!("Error creating JIT execution engine: {}", e);
            }
        }
        return;
    }

    let output_file_name = match &args.output_file {
        Some(name) => name.clone(),
        None => match &args.target {
            cli::TargetLanguage::Assembly => "p.s".into(),
            cli::TargetLanguage::Object => "p.o".into(),
            cli::TargetLanguage::Binary => "p.out".into(),
            _ => {
                eprintln!("Incompatible target language");
                exit(1);
            }
        },
    };

    if args.target == cli::TargetLanguage::Assembly {
        if let Err(e) = codegen.compile(&output_file_name, Assembly, &codegen.target_machine) {
            eprintln!("Error generating assembly: {}", e);
            exit(1);
        }
    }
    if args.target == cli::TargetLanguage::Object {
        if let Err(e) = codegen.compile(&output_file_name, Object, &codegen.target_machine) {
            eprintln!("Error generating object: {}", e);
            exit(1);
        }
    }
    if args.target == cli::TargetLanguage::Binary {
        if let Err(e) = codegen.generate_binary(&output_file_name, &target_triple) {
            eprintln!("Error generating binary: {}", e);
            exit(1);
        }
    }

    if args.run {
        let output_path = args.output_file.unwrap_or("p.out".into());
        let absolute_path = output_path.canonicalize().unwrap_or_else(|_| {
            eprintln!("Failed to get absolute path");
            exit(1);
        });
        if let Err(e) = execute_binary(&absolute_path) {
            eprintln!("{}", e);
            exit(1);
        }
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
        _ => Box::new(stdout()),
    };

    match args.format {
        cli::TableFormat::Plaintext => write!(out_handle, "{}", table.display_plain()),
        cli::TableFormat::Markdown => write!(out_handle, "{}", table.display_markdown()),
        cli::TableFormat::Rust => write!(out_handle, "{}", table.display_rust(false)),
    }
    .expect("Error while printing table");
}

fn symbol_table_example() {
    let root = init_symbol_table();
    let node = root.clone();
    node.borrow_mut().insert_symbol(
        1,
        SymbolTableElement {
            symbol: Symbol::Function(),
            name: String::from("main"),
            symbol_type: Type::Any,
        },
    );
    node.borrow_mut().insert_symbol(
        2,
        SymbolTableElement {
            symbol: Symbol::Variable(),
            name: String::from("x"),
            symbol_type: Type::Any,
        },
    );

    let node = enter_scope(node);
    node.borrow_mut().insert_symbol(
        3,
        SymbolTableElement {
            symbol: Symbol::Parameter(),
            name: String::from("param1"),
            symbol_type: Type::Any,
        },
    );
    node.borrow_mut().insert_symbol(
        4,
        SymbolTableElement {
            symbol: Symbol::Variable(),
            name: String::from("y"),
            symbol_type: Type::Any,
        },
    );
    node.borrow_mut().insert_symbol(
        5,
        SymbolTableElement {
            symbol: Symbol::Function(),
            name: String::from("helper"),
            symbol_type: Type::Any,
        },
    );

    let node = enter_scope(node);
    node.borrow_mut().insert_symbol(
        6,
        SymbolTableElement {
            symbol: Symbol::Function(),
            name: String::from("nested"),
            symbol_type: Type::Any,
        },
    );
    node.borrow_mut().insert_symbol(
        7,
        SymbolTableElement {
            symbol: Symbol::Variable(),
            name: String::from("z"),
            symbol_type: Type::Any,
        },
    );

    let node = exit_scope(node);
    node.borrow_mut().insert_symbol(
        8,
        SymbolTableElement {
            symbol: Symbol::Function(),
            name: String::from("sibling"),
            symbol_type: Type::Any,
        },
    );

    let node = enter_scope(node);

    let node = get_scope(node, 0).unwrap();
    node.borrow_mut().insert_symbol(
        9,
        SymbolTableElement {
            symbol: Symbol::Function(),
            name: String::from("outer"),
            symbol_type: Type::Any,
        },
    );

    let node = enter_scope(node);
    node.borrow_mut().insert_symbol(
        10,
        SymbolTableElement {
            symbol: Symbol::Variable(),
            name: String::from("w"),
            symbol_type: Type::Any,
        },
    );

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
