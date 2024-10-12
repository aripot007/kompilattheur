mod reader;

fn main() {
    println!("Hello, world!");

    let file_path = "readme.md".to_string();
    for c in reader::reader(file_path) {
        print!("{c}");
    }
    println!("Fin");
}
