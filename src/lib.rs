pub mod fs_helper;

pub fn print_help() {
    println!(
        "USAGE: 
        ./file_dedupe <path_to_directory>

        Arg 1 - path_to_directory - required. 
                This is the directory to scan for duplicate files. 
    "
    );
}
