use std::path::Path;

use crate::domain::Result;

mod flake_lock;
mod flake_nix;
mod sync_service;
mod sync_strategy;
mod update_service;

pub(crate) use flake_lock::{
    FlakeLock, InputReference, Locked, LockedRef, LockedSource, Node, Original, OriginalRef,
    OriginalSource, RootNode,
};
#[allow(clippy::module_name_repetitions)]
pub(crate) use flake_nix::FlakeNix;
pub use sync_service::{SyncService, SyncServiceImpl};
pub(crate) use sync_strategy::SyncStrategy;

pub trait Flake {
    fn load_lock(&self) -> Result<FlakeLock>;
    fn load_lock_from<P: AsRef<Path>>(&self, p: P) -> Result<FlakeLock>;
    fn load_from<P: AsRef<Path>>(&self, p: P) -> Result<FlakeNix>;

    fn write<P: AsRef<Path>>(&self, p: P, flake: &FlakeNix) -> Result<()>;

    fn override_input<P: AsRef<Path>>(&self, p: P, input: &str, url: &str) -> Result<()>;
}
