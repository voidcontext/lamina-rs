use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
    #[clap(short, long, action)]
    pub debug: bool,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prints the last modified date/time of the flake inputs
    LastModified,
    /// Syncs input with another flake
    Sync {
        with_flake: PathBuf,
        src_input_name: String,
        dst_input_name: String,
    },
}
