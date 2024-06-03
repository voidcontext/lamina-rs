use crate::domain::commands::SyncInputNames;

#[derive(Debug, PartialEq)]
pub enum SyncStrategy<'a> {
    LockOnly {
        lock_url: String,
        input_names: &'a SyncInputNames,
    },
    FlakeNixAndLock {
        lock_url: String,
        input_names: &'a SyncInputNames,
    },
}

impl<'a> SyncStrategy<'a> {
    #[must_use]
    pub(crate) fn lock_only(lock_url: String, input_names: &'a SyncInputNames) -> SyncStrategy<'a> {
        Self::LockOnly {
            lock_url,
            input_names,
        }
    }

    #[must_use]
    pub(crate) fn flake_nix_and_lock(
        lock_url: String,
        input_names: &'a SyncInputNames,
    ) -> SyncStrategy<'a> {
        Self::FlakeNixAndLock {
            lock_url,
            input_names,
        }
    }
}
