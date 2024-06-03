pub mod commands;
pub mod console;
pub mod fs;
pub mod nix;

use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("io error")]
    IOError(#[from] io::Error),
    #[error("flake.lock invalid: {reason}")]
    InvalidFlakeLock { reason: String },
    #[error("sync error: {:?}", .0)]
    SyncError(String),
    #[error("nix parser error: {:?}", .0)]
    NixParserError(String),
    #[error("an error happened: {:?}", .0)]
    Error(String),
}

pub type Result<T> = std::result::Result<T, Error>;
