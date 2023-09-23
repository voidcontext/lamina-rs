pub mod file;
pub mod flake_lock;
#[allow(clippy::module_name_repetitions)]
pub mod flake_nix;
pub mod process;

#[derive(Debug, PartialEq)]
pub enum SyncInputNames {
    SourceAndDestination { source: String, destination: String },
    Same { input_name: String },
}

impl SyncInputNames {
    #[must_use]
    pub fn source_and_destination(source: String, destination: String) -> Self {
        if source == destination {
            Self::Same { input_name: source }
        } else {
            Self::SourceAndDestination {
                source,
                destination,
            }
        }
    }

    #[must_use]
    pub fn same(input_name: String) -> Self {
        Self::Same { input_name }
    }

    #[must_use]
    pub fn source(&self) -> &String {
        match self {
            Self::SourceAndDestination {
                source,
                destination: _,
            } => source,
            Self::Same { input_name } => input_name,
        }
    }

    #[must_use]
    pub fn destination(&self) -> &String {
        match self {
            Self::SourceAndDestination {
                source: _,
                destination,
            } => destination,
            Self::Same { input_name } => input_name,
        }
    }
}

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
    pub fn lock_only(lock_url: String, input_names: &'a SyncInputNames) -> SyncStrategy<'a> {
        Self::LockOnly {
            lock_url,
            input_names,
        }
    }

    #[must_use]
    pub fn flake_nix_and_lock(
        lock_url: String,
        input_names: &'a SyncInputNames,
    ) -> SyncStrategy<'a> {
        Self::FlakeNixAndLock {
            lock_url,
            input_names,
        }
    }
}
