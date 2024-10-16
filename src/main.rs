mod reader;
mod lexer;

fn main() {
    
    let file_path = "test_programs/hello_world.smolpp".to_string();
    let lexer = lexer::new(reader::new(file_path));

    for token in lexer {
        print!("{} ", token);
    }

    print!("\n");
}
