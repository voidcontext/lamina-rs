use crate::domain;

pub trait Console {
    fn println<S: AsRef<str>>(&self, s: S) -> domain::Result<()>;
}
