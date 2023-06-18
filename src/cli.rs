use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Info,
}
