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
        src_flake: PathBuf,
        /// Name of the input in the source flake
        src_input_name: String,
        /// Path to the destination flake, current dir if not provided
        dst_flake: Option<PathBuf>,
        /// Name of the input in the destination flake, same as the SRC_INPUT_NAME if not provided.
        /// When this argument is set, DST_FLAKE needs to be set too.
        dst_input_name: Option<String>,
    },
    /// Syncs multiple inputs with another flake, inputs must have matching names
    BatchSync {
        /// Path to the source flake
        src_flake: PathBuf,
        /// Path to the destination flake
        dst_flake: PathBuf,
        /// Name of the inputs that will be synced
        inputs: Vec<String>,
    },
    /// Prints the last modified date/time of the flake inputs
    LastModified,
}
