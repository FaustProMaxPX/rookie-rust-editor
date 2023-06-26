use std::{fs, io};

use crate::Row;

/// we need a structure to represent the document the user is editing
/// and a vector of row should be included

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub filename: Option<String>,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, io::Error> {
        let contents = fs::read_to_string(filename)?;
        let rows = contents.lines().map(Row::from).collect();
        Ok(Self {
            rows,
            filename: Some(filename.to_string()),
        })
    }

    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }
}
