use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "keepass-dump-extrator")]
pub struct Args {
    /// The memory dump file to search
    pub input: PathBuf,

    /// The format to print the results in
    #[clap(short, long, default_value = "found")]
    pub format: Format,
}

#[derive(ValueEnum, Clone)]
pub enum Format {
    /// Directly print all hints about the password
    Found,
    /// Summarize the hints into the full size, leaving gaps for unknown characters
    Gaps,
    /// Print all possible permutations of the password, intended as a wordlist
    All,
    /// Write the raw results with all found information, intended for further processing
    /// (count, position, character)
    Raw,
}
