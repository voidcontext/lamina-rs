use std::path::Path;

use crate::nix::file::flake_lock::FlakeLock;
use crate::nix::process;
use crate::nix::{file, SyncInputNames};
use crate::nix::{flake_lock, flake_nix, SyncStrategy};

pub fn sync(src_path: &Path, dst_path: &Path, input_names: &SyncInputNames) -> anyhow::Result<()> {
    let source_flake_lock = FlakeLock::try_from(src_path)?;
    let destination_flake_lock = FlakeLock::try_from(dst_path)?;

    let source_rev = source_flake_lock
        .locked_rev_of(input_names.source())
        .ok_or_else(|| {
            anyhow::Error::msg(format!(
                "{} doesn't have a revision at source",
                input_names.source()
            ))
        })?;

    let destination_rev = destination_flake_lock
        .locked_rev_of(input_names.destination())
        .ok_or_else(|| {
            anyhow::Error::msg(format!(
                "{} doesn't have a revision at destination",
                input_names.destination()
            ))
        })?;

    log::debug!(
        "destination rev of {} is: {}",
        input_names.destination(),
        &*destination_rev
    );
    log::debug!(
        "source rev of {} is: {}",
        input_names.source(),
        &*source_rev
    );

    let syns_strategy =
        flake_lock::sync_strategy(&source_flake_lock, &destination_flake_lock, input_names)?;

    let dst_dir = if dst_path.is_dir() {
        dst_path
    } else {
        dst_path.parent().unwrap()
    };

    match syns_strategy {
        SyncStrategy::LockOnly {
            lock_url,
            input_names,
        } => process::override_input(input_names.destination(), &lock_url, dst_dir),
        SyncStrategy::FlakeNixAndLock {
            lock_url,
            input_names,
        } => {
            let source_flake_nix = file::flake_nix::read_to_string(src_path)?;
            let destination_flake_nix = file::flake_nix::read_to_string(dst_path)?;
            let modified_flake_nix =
                flake_nix::sync(&source_flake_nix, &destination_flake_nix, input_names)?;
            file::flake_nix::write(&dst_dir, &modified_flake_nix)?;
            process::override_input(input_names.destination(), &lock_url, dst_dir)
        }
    }
}
