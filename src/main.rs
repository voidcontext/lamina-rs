use std::env::current_dir;

use crate::cli::Args;
use clap::Parser;
use cli::Command;
use lamina::{
    console::OsConsole,
    domain::{
        self,
        commands::{self, SyncInputNames},
    },
    fs::OsFileSystem,
    nix::{Flake, FlakeLockMapperImpl},
};
use log::LevelFilter::{Debug, Info};
use simple_logger::SimpleLogger;

mod cli;

fn main() -> lamina::domain::Result<()> {
    let args = Args::parse();

    let log_level = if args.debug { Debug } else { Info };
    SimpleLogger::new().with_level(log_level).init().unwrap();

    match args.command {
        Command::LastModified => {
            let fs = OsFileSystem {};
            let lock_mapper = FlakeLockMapperImpl {};

            let console = OsConsole {};

            let flake = Flake::new(fs, lock_mapper);

            commands::last_modified(&flake, &console)
        }
        Command::Sync {
            src_flake,
            src_input_name,
            dst_flake,
            dst_input_name,
        } => {
            let fs = OsFileSystem {};
            let lock_mapper = FlakeLockMapperImpl {};

            let flake = Flake::new(fs, lock_mapper);
            let sync_service = domain::nix::SyncServiceImpl {};
            commands::sync(
                &src_flake,
                &(dst_flake
                    .unwrap_or_else(|| current_dir().expect("Couldn't determine the current dir"))),
                &[SyncInputNames::source_and_destination(
                    src_input_name.clone(),
                    dst_input_name.unwrap_or(src_input_name),
                )],
                &flake,
                &sync_service,
            )
        }
        Command::BatchSync {
            src_flake,
            dst_flake,
            inputs,
        } => {
            let fs = OsFileSystem {};
            let lock_mapper = FlakeLockMapperImpl {};

            let flake = Flake::new(fs, lock_mapper);
            let sync_service = domain::nix::SyncServiceImpl {};
            commands::sync(
                &src_flake,
                &dst_flake,
                &(inputs
                    .iter()
                    .map(|name| SyncInputNames::same(name.clone()))
                    .collect::<Vec<_>>()),
                &flake,
                &sync_service,
            )
        }
    }
}
