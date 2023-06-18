use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Prints the last modified date/time of the flake inputs
    LastModified,
}
