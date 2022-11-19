use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Write;
use std::{fs, io, path::PathBuf};

#[derive(Serialize, Clone)]
pub struct FileInfo {
    pub hash: String,
    pub file_names: Vec<String>,
    pub count: i32,
    pub file_size: u64,
    pub dupe_size: u64,
}

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

pub fn write_output(f_name: &str, payload: Vec<FileInfo>) -> u64 {
    println!("Creating output file {}", f_name);
    let mut out_file = File::create(f_name).unwrap();

    out_file.write(b"[").unwrap();

    let mut dupe_space = 0;
    let mut idx = 1;
    let max_index = payload.len();
    for x in payload {
        let output = serde_json::to_string(&x).unwrap();
        out_file.write(output.as_bytes()).unwrap();

        if idx != max_index {
            out_file.write(b",\n").unwrap();
        }

        dupe_space += x.file_size * u64::try_from(x.file_names.len() - 1).unwrap();
        idx += 1;
    }
    out_file.write(b"]").unwrap();

    dupe_space
}
