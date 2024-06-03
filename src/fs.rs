use std::{
    env, fs,
    path::{Path, PathBuf},
};

use crate::domain::{self, fs::FileSystem};

pub fn ensure_file(path: &Path, file_name: &str) -> domain::Result<PathBuf> {
    let mut path = path.to_path_buf();
    if path.is_dir() {
        path.push(file_name);
    }

    let result_file_name = path
        .as_path()
        .file_name()
        .ok_or(domain::Error::Error(format!(
            "Cannot determine file name of path '{path:?}'"
        )))?;

    if result_file_name == file_name {
        Ok(path)
    } else {
        Err(domain::Error::Error(format!(
            "Path '{path:?}' doesn't match file name: {file_name:?}",
        )))
    }
}

pub struct OsFileSystem {}

impl FileSystem for OsFileSystem {
    fn read_to_string<P: AsRef<Path>>(&self, p: P) -> domain::Result<String> {
        let str = fs::read_to_string(p)?;

        Ok(str)
    }

    fn current_dir(&self) -> domain::Result<PathBuf> {
        let current_dir = env::current_dir()?;

        Ok(current_dir)
    }

    fn write<P: AsRef<Path>>(&self, p: P, str: &str) -> domain::Result<()> {
        fs::write(p, str)?;

        Ok(())
    }
}
