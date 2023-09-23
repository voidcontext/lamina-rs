use std::{env::current_dir, path::Path};

use crate::nix::{
    file::{self, flake_lock::FlakeLock},
    flake_lock, flake_nix, process, SyncInputNames, SyncStrategy,
};

pub fn batch_sync(source: &Path, inputs: &[SyncInputNames]) -> anyhow::Result<()> {
    let source_flake_lock = FlakeLock::try_from(source)?;
    let destination_flake_lock = FlakeLock::try_from(current_dir()?.as_path())?;

    let strategies = inputs
        .iter()
        .map(|input_name| {
            let source_rev = source_flake_lock
                .locked_rev_of(input_name.source())
                .ok_or_else(|| {
                    anyhow::Error::msg(format!(
                        "{} doesn't have a revision at source",
                        input_name.source()
                    ))
                })?;

            let destination_rev = destination_flake_lock
                .locked_rev_of(input_name.destination())
                .ok_or_else(|| {
                    anyhow::Error::msg(format!(
                        "{} doesn't have a revision at destination",
                        input_name.source()
                    ))
                })?;

            log::debug!(
                "destination rev of {} is: {}",
                input_name.destination(),
                &*destination_rev
            );
            log::debug!("source rev of {} is: {}", input_name.source(), &*source_rev);

            flake_lock::sync_strategy(&source_flake_lock, &destination_flake_lock, input_name)
        })
        .collect::<anyhow::Result<Vec<SyncStrategy>>>()?;

    let source_flake_nix = file::flake_nix::read_to_string(source)?;
    let destination_flake_nix = file::flake_nix::read_to_string(&current_dir()?)?;

    let modified_flake_nix_content =
        strategies
            .iter()
            .fold(Ok(destination_flake_nix), |content, strategy| {
                content.and_then(|content_str| match strategy {
                    SyncStrategy::LockOnly {
                        lock_url: _,
                        input_names: _,
                    } => Ok(content_str),
                    SyncStrategy::FlakeNixAndLock {
                        lock_url: _,
                        input_names,
                    } => flake_nix::sync(&source_flake_nix, &content_str, input_names),
                })
            })?;

    file::flake_nix::write(&current_dir()?, &modified_flake_nix_content)?;

    strategies
        .iter()
        .map(|strategy| match strategy {
            SyncStrategy::LockOnly {
                lock_url,
                input_names,
            }
            | SyncStrategy::FlakeNixAndLock {
                lock_url,
                input_names,
            } => process::override_input(input_names.destination(), lock_url),
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    Ok(())
}
