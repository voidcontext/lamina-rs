use std::env::current_dir;
use std::path::Path;

use crate::nix::file;
use crate::nix::file::flake_lock::FlakeLock;
use crate::nix::process;
use crate::nix::{flake_lock, flake_nix, SyncStrategy};

pub fn sync(src_path: &Path, src_input_name: &str, dst_input_name: &str) -> anyhow::Result<()> {
    let source_flake_lock = FlakeLock::try_from(src_path)?;
    let destination_flake_lock = FlakeLock::try_from(current_dir()?.as_path())?;

    let source_rev = source_flake_lock
        .locked_rev_of(src_input_name)
        .ok_or_else(|| {
            anyhow::Error::msg(format!(
                "{src_input_name} doesn't have a revision at source"
            ))
        })?;

    let destination_rev = destination_flake_lock
        .locked_rev_of(dst_input_name)
        .ok_or_else(|| {
            anyhow::Error::msg(format!(
                "{dst_input_name} doesn't have a revision at destination"
            ))
        })?;

    log::debug!(
        "destination rev of {dst_input_name} is: {}",
        &*destination_rev
    );
    log::debug!("source rev of {src_input_name} is: {}", &*source_rev);

    let syns_strategy = flake_lock::sync_strategy(
        &source_flake_lock,
        src_input_name,
        &destination_flake_lock,
        dst_input_name,
    )?;

    match syns_strategy {
        SyncStrategy::LockOnly(new_url) => process::override_input(dst_input_name, &new_url),
        SyncStrategy::FlakeNixAndLock(new_url) => {
            let source_flake_nix = file::flake_nix::read_to_string(src_path)?;
            let destination_flake_nix = file::flake_nix::read_to_string(&current_dir()?)?;
            let modified_flake_nix = flake_nix::sync(
                &source_flake_nix,
                src_input_name,
                &destination_flake_nix,
                dst_input_name,
            )?;
            file::flake_nix::write(&current_dir()?, &modified_flake_nix)?;
            process::override_input(dst_input_name, &new_url)
        }
    }
}
