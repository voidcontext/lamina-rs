use anyhow::Context;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub fn read_to_string(path: &Path) -> anyhow::Result<String> {
    fs::read_to_string(ensure_flake_nix_path(path))
        .with_context(|| format!("Failed to read {:?}", ensure_flake_nix_path(path).to_str()))
}

pub fn write(path: &Path, content: &str) -> anyhow::Result<()> {
    fs::write(ensure_flake_nix_path(path), content)
        .with_context(|| format!("Failed to write {:?}", ensure_flake_nix_path(path).to_str()))
}

fn ensure_flake_nix_path(path: &Path) -> PathBuf {
    super::ensure_file(path, "flake.nix")
}
