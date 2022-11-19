pub mod fs_helper;

pub fn print_help() {
    println!(
        "USAGE:
        ./file_dedupe <path_to_directory> [<report_file_name>]

        Arg 1 - Required - path_to_directory
                This is the directory to scan for duplicate files.

        Arg 2 - Optional - report_file_name
                The name of the file to create, will default to out.json.
                This output file is the details on the duplicates discovered.
    "
    );
}
