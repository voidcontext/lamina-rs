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
        Command::LastModified => {
            commands::last_modified();
            Ok(())
        }
        Command::Sync {
            dst_input,
            with_flake,
            src_input,
        } => commands::sync(&dst_input, &with_flake, &src_input),
    }
}
