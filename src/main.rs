use crossbeam_channel::unbounded;
use std::collections::HashMap;
use std::env;
use std::process;

use file_dedupe::fs_helper::{gen_hash, pull_all_files, write_output, FileInfo};
use file_dedupe::print_help;

const OUT_FILE: &str = "./out.json";

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
    if dir_to_scan.to_lowercase().contains("help") {
        print_help();
        process::exit(0);
    }

    let all_files = pull_all_files(dir_to_scan);
    let (s, r) = unbounded();
    let mut handles = Vec::new();
    let mut total_size: u64 = 0;
    println!("Files found {}", all_files.len());

    // Loop through all the files and gen hashes async
    for x in all_files {
        total_size += match x.metadata() {
            Ok(x) => x.len(),
            Err(_) => 0,
        };

        let send_channel = s.clone();
        let handle = tokio::spawn(async move {
            let x = gen_hash(&x).await;
            send_channel.send(1).unwrap();
            x
        });
        handles.push(handle);
    }

    // Listen to our channel until we have all the messages
    let mut cnt = 1.0;
    let tot_cnt = handles.len() as f64;
    while cnt != tot_cnt {
        let msgs: Vec<i32> = r.try_iter().collect();
        for msg in msgs {
            cnt += msg as f64;
        }
        let prcnt = 100 as f64 * (cnt / tot_cnt);
        print!("\rProcessing {}/{} ({:.2}%)", cnt, tot_cnt, prcnt);
    }

    // Gather the response from our async processes
    let mut file_hashes: HashMap<String, FileInfo> = HashMap::new();
    for handle in handles {
        let (file_name, file_size, hash) = handle.await.unwrap();
        if file_hashes.contains_key(&hash) {
            let payload: &mut FileInfo = file_hashes.get_mut(&hash).unwrap();

            payload.file_names.push(file_name);
            payload.count += 1;
            payload.dupe_size += payload.file_size;
        } else {
            let payload = FileInfo {
                hash: hash.clone(),
                file_names: vec![file_name],
                count: 1,
                file_size,
                dupe_size: 0,
            };
            file_hashes.insert(hash.clone(), payload);
        }
    }

    println!("\nAll hashes generated!\n\n");
    let sorted_output = sort_files(file_hashes);
    let dupe_space = write_output(output_file_name, sorted_output);

    println!("Total size of dir (bytes): {total_size}");
    println!("Total size of duplicate files (bytes): {dupe_space}");
}

fn sort_files(unsorted: HashMap<String, FileInfo>) -> Vec<FileInfo> {
    let mut sorted_output: Vec<FileInfo> = Vec::new();
    for (_, v) in unsorted.iter() {
        if v.count <= 1 {
            // There aren't dupes, skip
            continue;
        }

        // This is a dupe - obtain ownership
        let value = v.to_owned();

        let out_len = sorted_output.len();
        if out_len == 0 {
            // First item - place it in the empty list and move on
            sorted_output.push(value);
            continue;
        }

        let mut idx = 0;
        while idx < out_len {
            let curr = sorted_output[idx].dupe_size;

            if value.dupe_size >= curr {
                sorted_output.insert(idx, value);
                break;
            } else if idx == out_len - 1 {
                sorted_output.push(value);
                break;
            }

            // Not at the end of the list and not greater than anything just yet
            idx += 1;
        }
    }

    sorted_output
}
