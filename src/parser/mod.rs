mod v1;
mod v2;

pub fn main() {
    let arg = std::env::args_os().nth(1);
    if let Some(path) = arg {
        let file = std::fs::read_to_string(path).expect("failed to read file");
        println!("{file}");
        v1::parse_file(&file);
        // v2::parse_file(&file);
    } else {
        v1::run_stdin();
    }
}
