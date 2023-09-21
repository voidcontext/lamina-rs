use std::path::{Path, PathBuf};

pub mod flake_lock;
pub mod flake_nix;

pub(self) fn ensure_file(path: &Path, file_name: &str) -> PathBuf {
    let mut path = path.to_path_buf();
    if path.is_dir() {
        path.push(file_name);
    }
    path
}
