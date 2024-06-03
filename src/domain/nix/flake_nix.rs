pub struct FlakeNix {
    as_string: String,
}

impl FlakeNix {
    pub(crate) fn new(content: String) -> Self {
        Self { as_string: content }
    }

    pub(crate) fn as_string(&self) -> String {
        self.as_string.clone()
    }
}
