mod reader;
mod lexer;
use std::env;

fn main() {

    let mut args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        // Le programme final terminera ici, mais pour le dev je laisse un fichier par défaut
        args.push(String::from("test_programs/hello_world.smolpp"));
    }
    
    let file_path = args[1].clone();
    let lexer = lexer::new(reader::new(file_path));

    for token in lexer {
        print!("{} ", token);
    }

    print!("\n");
}
