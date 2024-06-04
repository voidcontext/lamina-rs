use std::{
    env::{self, current_dir},
    path::PathBuf,
};

use crate::cli::Args;
use clap::Parser;
use cli::Command;
use lamina::{
    console::OsConsole,
    domain::{
        self,
        commands::{self, SyncInputNames},
        nix::UpdateServiceImpl,
    },
    fs::OsFileSystem,
    git::{GitRepositoryConfig, GitRepositoryLibGit},
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
        Command::Outdated => {
            let fs = OsFileSystem {};
            let lock_mapper = FlakeLockMapperImpl {};

            let console = OsConsole {};

            let flake = Flake::new(fs, lock_mapper);

            let mut config_dir = PathBuf::from(env::var("HOME").unwrap());
            config_dir.push(".cache/lamina/repos");

            let config = GitRepositoryConfig::new(true, config_dir);
            let git_repo = GitRepositoryLibGit::new(config);

            let update_service = UpdateServiceImpl::new(git_repo);

            commands::outdated(&flake, &console, &update_service)
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
