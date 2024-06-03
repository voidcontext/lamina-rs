use std::path::Path;

use crate::domain::{
    self,
    nix::{Flake, SyncService, SyncStrategy},
    Result,
};

use super::SyncInputNames;

pub fn sync<F: Flake, S: SyncService>(
    source: &Path,
    destination: &Path,
    inputs: &[SyncInputNames],
    flake: &F,
    sync_service: &S,
) -> Result<()> {
    let source_flake_lock = flake.load_lock_from(source)?;
    let destination_flake_lock = flake.load_lock_from(destination)?;

    let strategies = inputs
        .iter()
        .map(|input_name| {
            let source_rev = source_flake_lock
                .nodes
                .get(input_name.source())
                .map(|n| n.locked.rev.clone())
                .ok_or_else(|| {
                    domain::Error::SyncError(format!(
                        "{} doesn't have a revision at source",
                        input_name.source()
                    ))
                })?;

            let destination_rev = destination_flake_lock
                .nodes
                .get(input_name.destination())
                .map(|n| n.locked.rev.clone())
                .ok_or_else(|| {
                    domain::Error::SyncError(format!(
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

            sync_service.sync_strategy(&source_flake_lock, &destination_flake_lock, input_name)
        })
        .collect::<Result<Vec<SyncStrategy>>>()?;

    let source_flake_nix = flake.load_from(source)?;
    let destination_flake_nix = flake.load_from(destination)?;

    let modified_flake_nix_content =
        strategies
            .iter()
            .try_fold(destination_flake_nix, |result, strategy| match strategy {
                SyncStrategy::LockOnly {
                    lock_url: _,
                    input_names: _,
                } => Ok(result),
                SyncStrategy::FlakeNixAndLock {
                    lock_url: _,
                    input_names,
                } => sync_service.sync(&source_flake_nix, &result, input_names),
            })?;

    flake.write(destination, &modified_flake_nix_content)?;

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
            } => flake.override_input(destination, input_names.destination(), lock_url),
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(())
}
