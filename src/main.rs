use std::fs;

use clap::Parser;

use keepass_dump_extractor::cli::Args;

fn main() {
    let args = Args::parse();
    let bytes = fs::read(args.input).expect("failed to read file");
    let leak = keepass_dump_extractor::find_leaks(&bytes);

    println!("{:?}", leak);
}
