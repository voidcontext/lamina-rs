pub mod file;
pub mod flake_lock;
#[allow(clippy::module_name_repetitions)]
pub mod flake_nix;
pub mod process;

#[derive(Debug, PartialEq)]
pub enum SyncStrategy {
    LockOnly(String),
    FlakeNixAndLock(String),
}
