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
    /// Syncs input with another flake
    Sync {
        /// Path to the source flake
        #[arg(long)]
        src: PathBuf,
        /// Path to the destination flake
        #[arg(long)]
        dst: Option<PathBuf>,
        /// Name of the input in the source flake
        src_input_name: String,
        /// Name of the input in the destination flake
        dst_input_name: String,
    },
    /// Syncs multiple inputs with another flake, the inputs need to have matching name
    BatchSync {
        /// Path to the source flake
        #[arg(long)]
        src: PathBuf,
        /// Path to the destination flake
        #[arg(long)]
        dst: Option<PathBuf>,
        /// Name of the inputs that will be synced
        inputs: Vec<String>,
    },
    /// Prints the last modified date/time of the flake inputs
    LastModified,
}
