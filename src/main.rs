use crate::cli::Args;
use clap::Parser;
use cli::Command;

mod cli;
mod commands;
mod nix;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args.command {
        Command::LastModified => {
            commands::last_modified();
            Ok(())
        }
        Command::Sync {
            dst_input,
            with_flake,
            src_input,
        } => commands::sync(&dst_input, with_flake, &src_input),
    }
}
