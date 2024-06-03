mod last_modified;
mod sync;

pub use last_modified::last_modified;
pub use sync::sync;

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
