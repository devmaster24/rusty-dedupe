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
    file_size: u64,
    dupe_size: u64,
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
    let output_file_name = match args.get(2) {
        Some(s) => s,
        None => OUT_FILE,
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
        let (file_name, file_size, hash) = handle.await.unwrap();

        if file_hashes.contains_key(&hash) {
            let mut payload: FileInfo = file_hashes.get(&hash).unwrap().clone();

            payload.file_names.push(file_name);
            payload.count += 1;
            payload.dupe_size = (payload.count as u64 - 1) * payload.file_size;

            file_hashes.insert(hash.clone(), payload);
        } else {
            let payload = FileInfo {
                hash: hash.clone(),
                file_names: vec![file_name],
                count: 1,
                file_size: file_size,
                dupe_size: 0,
            };
            file_hashes.insert(hash.clone(), payload);
        }
    }

    // Sort hashmap
    let mut sorted_output: Vec<&FileInfo> = Vec::new();
    for (_, value) in file_hashes.iter() {
        if value.count <= 1 {
            // There aren't dupes, skip
            continue;
        }

        if sorted_output.len() == 0 {
            sorted_output.push(value);
            continue;
        }

        let mut idx = 0;
        while idx < sorted_output.len() {
            if idx + 1 == sorted_output.len() {
                sorted_output.push(value);
                break;
            }

            let curr = sorted_output[idx].dupe_size;
            let next = sorted_output[idx + 1].dupe_size;

            if curr == next {
                // If they are equal, doesn't matter just plop it in there
                sorted_output.insert(idx, value);
                break;
            } else if curr > next {
                // Next one is smaller, this is the proper place
                sorted_output.insert(idx, value);
                break;
            }
            // Keep on looking
            idx += 1;
        }
    }

    println!("Creating output file {}", output_file_name);
    let mut out_file = File::create(output_file_name).unwrap();
    out_file.write(b"[").unwrap();

    let mut dupe_space = 0;
    let mut idx = 1;
    let max_index = sorted_output.len();

    for x in sorted_output {
        let output = serde_json::to_string(&x).unwrap();
        out_file.write(output.as_bytes()).unwrap();

        if idx != max_index {
            out_file.write(b",\n").unwrap();
        }

        dupe_space += x.file_size * u64::try_from(x.file_names.len() - 1).unwrap();

        idx += 1;
    }
    out_file.write(b"]").unwrap();

    println!("Total size of dir (bytes): {total_size}");
    println!("Total size of duplicate files (bytes): {dupe_space}");
}
