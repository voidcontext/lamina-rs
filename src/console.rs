use std::io::{self, Write};

use crate::domain;

#[allow(clippy::module_name_repetitions)]
pub struct OsConsole {}

impl domain::console::Console for OsConsole {
    fn println<S: AsRef<str>>(&self, s: S) -> domain::Result<()> {
        io::stdout().write_all(s.as_ref().as_bytes())?;
        Ok(())
    }
}
