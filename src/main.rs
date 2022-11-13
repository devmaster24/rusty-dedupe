use file_dedupe::print_help;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::process;
use tokio;

use file_dedupe::fs_helper::{gen_hash, pull_all_files};

const OUT_FILE: &str = "./out.txt";

#[derive(Serialize, Clone)]
struct FileInfo {
    hash: String,
    file_names: Vec<String>,
    count: i32,
}

#[tokio::main]
async fn main() {
    // Expect first arg to be the directory to parse - graceful exit if missing
    let args: Vec<String> = env::args().collect();
    let dir_to_scan = match args.get(1) {
        Some(s) => s,
        None => {
            println!("ERROR - Missing required argument!\n");
            print_help();
            process::exit(1);
        }
    };

    let all_files = pull_all_files(dir_to_scan);
    println!("Files found {}", all_files.len());

    let mut handles = Vec::new();

    let mut total_size: u64 = 0;
    for x in all_files {
        total_size += match x.metadata() {
            Ok(x) => x.len(),
            Err(_) => 0,
        };

        let handle = tokio::spawn(async move {
            let x = gen_hash(&x).await;
            x
        });
        handles.push(handle);
    }

    let mut file_hashes: HashMap<String, FileInfo> = HashMap::new();

    for handle in handles {
        let (file_name, hash) = handle.await.unwrap();

        if file_hashes.contains_key(&hash) {
            let mut payload: FileInfo = file_hashes.get(&hash).unwrap().clone();

            payload.file_names.push(file_name);
            payload.count += 1;

            file_hashes.insert(hash.clone(), payload);
        } else {
            let payload = FileInfo {
                hash: hash.clone(),
                file_names: vec![file_name],
                count: 1,
            };
            file_hashes.insert(hash.clone(), payload);
        }
    }

    println!("Creating output file {}", OUT_FILE);
    let mut out_file = File::create(OUT_FILE).unwrap();
    for (_, payload) in file_hashes.iter() {
        if payload.count > 1 {
            let output = serde_json::to_string(&payload).unwrap();
            out_file.write(output.as_bytes()).unwrap();
            out_file.write(b"\n").unwrap();
        }
    }

    println!("Total size: {total_size}");
}
