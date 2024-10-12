mod reader;

fn main() {
    println!("Hello, world!");

    let file_path = "readme.md".to_string();
    let mut file_chars = reader::reader(file_path);
    while let Some(c) = file_chars.next() {
        print!("{}", c);
    }
    println!("Fin");
}
