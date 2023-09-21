use crate::cli::Args;
use clap::Parser;
use cli::Command;
use lamina::commands;
use log::LevelFilter::{Debug, Info};
use simple_logger::SimpleLogger;

mod cli;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let log_level = if args.debug { Debug } else { Info };
    SimpleLogger::new().with_level(log_level).init().unwrap();

    match args.command {
        Command::LastModified => commands::last_modified(),
        Command::Sync {
            dst_input_name,
            with_flake,
            src_input_name,
        } => commands::sync(&with_flake, &src_input_name, &dst_input_name),
    }
}
