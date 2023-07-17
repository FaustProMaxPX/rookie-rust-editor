pub struct FileType {
    name: String,
    hl_opts: HighlightingOptions,
}

/// this structure will hold a series of bool value representing the highlighting options
#[derive(Default, Clone, Copy)]
pub struct HighlightingOptions {
    // whether numbers need to be highlighted
    numbers: bool,
    strings: bool,
}

impl FileType {
    #[must_use]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[must_use]
    pub fn highlighting_opts(&self) -> HighlightingOptions {
        self.hl_opts
    }
}

impl Default for FileType {
    fn default() -> Self {
        Self {
            name: String::from("No file type"),
            hl_opts: HighlightingOptions::default(),
        }
    }
}

impl From<String> for FileType {
    fn from(filename: String) -> Self {
        if std::path::Path::new(&filename)
            .extension()
            .map_or(false, |ext| ext.eq_ignore_ascii_case("rs"))
        {
            return Self {
                name: "Rust".to_string(),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                },
            };
        }
        FileType::default()
    }
}

impl From<&str> for FileType {
    fn from(filename: &str) -> Self {
        Self::from(filename.to_string())
    }
}

impl HighlightingOptions {
    // we can just use self here instead of &self
    // because rust can deal with value faster if it's small enough
    // TODO: why?
    pub fn numbers(&self) -> bool {
        self.numbers
    }

    pub fn strings(&self) -> bool {
        self.strings
    }
}
