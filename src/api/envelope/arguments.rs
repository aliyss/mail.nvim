pub struct EnvelopeListArguments {
    page: Option<usize>,
    per_page: Option<usize>,
}

impl EnvelopeListArguments {
    #[must_use]
    pub fn new(page: Option<usize>, per_page: Option<usize>) -> Self {
        Self { page, per_page }
    }

    #[must_use]
    pub fn page(&self) -> Option<usize> {
        self.page
    }

    #[must_use]
    pub fn per_page(&self) -> Option<usize> {
        self.per_page
    }

    #[must_use]
    pub fn page_or_default(&self) -> usize {
        1.max(self.page.unwrap_or(1)) - 1
    }

    pub fn per_page_or_default(&self, f: Option<impl Fn() -> usize>) -> usize {
        1.max(self.per_page.unwrap_or_else(|| f.map_or(10, |func| func())))
    }
}

impl Default for EnvelopeListArguments {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(10),
        }
    }
}
