# Rusty File Duplicate Identifier

This project is intended locate duplicate files based on their SHA256 digest and create a JSON report of the duplicates.

The benefits of this approach is duplicate files will be located based on their content, not their name.

No files will be deleted by running this program, only an output file will be created.


# Usage

This program is designed to be executed as a command line program - there is no GUI at this point.

To run:
`./file_dedupe[.exe] <directory to scan> [<output file name>]`

Or:
`./file_dedupe[.exe] --help`


# Compiling

Run these steps on an arch linux based machine

## Linux
```sh
rustup target add x86_64-unknown-linux-musl
cargo build --target=x86_64-unknown-linux-musl --release
```

## Windows
```sh
sudo pacman -S mingw-w64-gcc
rustup target add x86_64-pc-windows-gnu
cargo build --target=x86_64-pc-windows-gnu --release
```
