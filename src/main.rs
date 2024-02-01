use std::fs;

use clap::Parser;

use keepass_dump_extractor::{cli::Args, find_leaks, print_formatted_leaks};

fn main() {
    let args = Args::parse();
    let bytes = fs::read(args.input).expect("Failed to read file");
    let leaks = find_leaks(&bytes);

    print_formatted_leaks(&leaks, args.format);
}
