mod reader;
mod lexer;

fn main() {
    println!("Hello, world!");

    let file_path = "readme.md".to_string();
    for c in reader::new(file_path) {
        print!("{c}");
    }
    println!("Fin");
}
