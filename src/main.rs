mod reader;
mod common;
mod lexer;
mod parser;
use lexer::lexer::Lexer;
use parser::generate_tree::generate_tree;
use std::env;

fn main() {

    let mut args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        // Le programme final terminera ici, mais pour le dev je laisse un fichier par défaut
        args.push(String::from("test_programs/hello_world.smolpp"));
    }
    
    let file_path = args[1].clone();
    let lexer = Lexer::new(reader::new(file_path.clone()));

    for token in lexer {
        print!("{} ", token);
    }

    print!("\n");
    let lexer = Lexer::new(reader::new(file_path));
    generate_tree(lexer);
}

