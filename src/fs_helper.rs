use sha2::{Digest, Sha256};
use std::{fs, io, path::PathBuf};

pub fn pull_all_files(dir: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    for file in fs::read_dir(&dir).unwrap() {
        let file_obj = file.unwrap().path();
        let file_name = file_obj.as_path().to_str().unwrap();

        if file_obj.is_dir() {
            let mut nested_files: Vec<PathBuf> = pull_all_files(&file_name);
            paths.append(&mut nested_files);
        } else {
            paths.push(file_obj);
        }
    }

    paths
}

pub async fn gen_hash(fp: &PathBuf) -> (String, u64, String) {
    let file_name = format!("{}", fp.display());
    let file_size = fp.metadata().unwrap().len();

    let mut data = match fs::File::open(&fp) {
        Ok(s) => s,
        Err(_) => {
            println!("Failed to read {}", fp.display());
            return (file_name, 0, "".to_string());
        }
    };

    let mut hasher = Sha256::new();
    let _ = io::copy(&mut data, &mut hasher);

    let hash = format!("{:X}", hasher.finalize());

    (file_name, file_size, hash)
}
