use crate::cli::Args;
use clap::Parser;
use cli::Command;

mod cli;
mod commands;
mod nix;

fn main() {
    let args = Args::parse();

    match args.command {
        Command::LastModified => commands::last_modified(),
    }
}
