use std::path::{Path, PathBuf};

use crate::domain::Result;

pub trait FileSystem {
    fn read_to_string<P: AsRef<Path>>(&self, p: P) -> Result<String>;

    fn write<P: AsRef<Path>>(&self, p: P, str: &str) -> Result<()>;

    fn current_dir(&self) -> Result<PathBuf>;
}
