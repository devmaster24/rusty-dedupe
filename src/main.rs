use std::fs;

fn main() {
    for file in fs::read_dir("TODO").unwrap() {
        let file_obj = file.unwrap().path();

        println!("Path - {}", file_obj.display());
        println!("Is Dir - {}", file_obj.is_dir());
        if !file_obj.is_dir() {
            println!("{}", file_obj.metadata().unwrap().len());
        }
    }
}
