use crate::cli::Args;
use clap::Parser;
use cli::Command;
use lamina::{
    commands::{self, batch_sync},
    nix::SyncInputNames,
};
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
        } => commands::sync(
            &with_flake,
            &SyncInputNames::source_and_destination(src_input_name, dst_input_name),
        ),
        Command::BatchSync { with_flake, inputs } => batch_sync(
            &with_flake,
            &inputs
                .into_iter()
                .map(SyncInputNames::same)
                .collect::<Vec<_>>(),
        ),
    }
}
