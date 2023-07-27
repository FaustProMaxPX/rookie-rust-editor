use std::fs::File;

pub struct FileType {
    name: String,
    hl_opts: HighlightingOptions,
}

/// this structure will hold a series of bool value representing the highlighting options
#[derive(Default, Clone, Debug)]
pub struct HighlightingOptions {
    // whether numbers need to be highlighted
    numbers: bool,
    strings: bool,
    characters: bool,
    comments: bool,
    primary_keys: Vec<String>,
    secondary_keys: Vec<String>,
}

impl FileType {
    #[must_use]
    pub fn name(&self) -> String {
        self.name.clone()
    }

    #[must_use]
    pub fn highlighting_opts(&self) -> &HighlightingOptions {
        &self.hl_opts
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

        if let Some(suffix) = std::path::Path::new(&filename).extension() {
            if suffix.to_str().is_none() {
                return FileType::default();
            }
            let suffix = suffix.to_str().unwrap();
            let kw_path = "src/highlightkeys/".to_string() + suffix + ".json";
            let mut primary_keys = vec![];
            let mut secondary_keys = vec![];
            let keywords: serde_json::Value =
                serde_json::from_reader(File::open(kw_path).unwrap()).unwrap();
            keywords[suffix]["primary_keys"]
                .as_array()
                .unwrap()
                .iter()
                .for_each(|key| {
                    primary_keys.push(key.as_str().unwrap().to_string());
                });
            keywords[suffix]["secondary_keys"]
                .as_array()
                .unwrap()
                .iter()
                .for_each(|key| {
                    secondary_keys.push(key.as_str().unwrap().to_string());
                });
            return Self {
                name: suffix.to_string(),
                hl_opts: HighlightingOptions {
                    numbers: true,
                    strings: true,
                    characters: true,
                    comments: true,
                    primary_keys,
                    secondary_keys,
                }
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

    pub fn characters(&self) -> bool {
        self.characters
    }

    pub fn comments(&self) -> bool {
        self.comments
    }

    pub fn primary_keys(&self) -> &Vec<String> {
        &self.primary_keys
    }

    pub fn secondary_keys(&self) -> &Vec<String> {
        &self.secondary_keys
    }
}

mod test {
    #[warn(unused_imports)]
    use crate::FileType;

    #[test]
    fn create_filetype() {
        let filetype = FileType::from("editor.rs");
        println!("{:#?}", filetype.highlighting_opts());
    }
}
