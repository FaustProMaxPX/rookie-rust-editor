use std::{
    fs,
    io::{self, Error, Write},
};

use crate::{FileType, Position, Row, SearchDirection};

/// we need a structure to represent the document the user is editing
/// and a vector of row should be included
/// `dirty` used to represent if the file has been modified since opened
#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub filename: Option<String>,
    dirty: bool,
    filetype: FileType,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self, io::Error> {
        let contents = fs::read_to_string(filename)?;
        let filetype = FileType::from(filename);
        let rows = contents
            .lines()
            .map(|line| {
                let mut row = Row::from(line);
                row.highlight(None, filetype.highlighting_opts());
                row
            })
            .collect();
        Ok(Self {
            rows,
            filename: Some(filename.to_string()),
            dirty: false,
            filetype,
        })
    }

    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.len() {
            return;
        }
        self.dirty = true;
        if c == '\n' {
            self.insert_newline(at);
            return;
        }
        let opts = self.filetype.highlighting_opts();
        if at.y == self.len() {
            let mut row = Row::default();
            row.insert(0, c);
            row.highlight(None, opts);
            self.rows.push(row);
        } else {
            let row = &mut self.rows[at.y];
            row.insert(at.x, c);
            row.highlight(None, opts);
        }
    }

    #[allow(clippy::integer_arithmetic)]
    pub fn delete(&mut self, at: &Position) {
        // if the cursor's y is greater than the number of rows
        // there is nothing to delete
        let len = self.len();
        if at.y >= len {
            return;
        }

        self.dirty = true;

        let opts = self.filetype.highlighting_opts();

        // if the cursor is at the end of line
        // then we should delete the next row, and append it to current line
        if at.x == self.rows[at.y].len() && at.y + 1 < len {
            let next_row = self.rows.remove(at.y + 1);
            let row = &mut self.rows[at.y];
            row.append(&next_row);
            row.highlight(None, opts);
        } else {
            let row = &mut self.rows[at.y];
            row.delete(at.x);
            row.highlight(None, opts);
        }
    }

    pub fn insert_newline(&mut self, at: &Position) {
        if at.y > self.len() {
            return;
        }

        if at.y == self.len() {
            self.rows.push(Row::default());
            return;
        }

        // if we want to insert a newline at the middle of the row
        // we should first split it at current cursor position
        // and the last part become the next new row
        #[allow(clippy::indexing_slicing)]
        let mut new_row = self.rows[at.y].split(at.x);
        let opts = self.filetype.highlighting_opts();
        self.rows[at.y].highlight(None, opts);
        new_row.highlight(None, opts);
        #[allow(clippy::integer_arithmetic)]
        self.rows.insert(at.y + 1, new_row);
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if let Some(filename) = &self.filename {
            let mut file = fs::File::create(filename)?;
            for row in &self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }
            self.filetype = FileType::from(filename.to_string());
            self.dirty = false;
            self.highlight(None);
        }
        Ok(())
    }

    /// find a segment equal to `query`.
    /// at and direction represent the position of start point and the direction of searching
    #[must_use]
    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }
        let mut position = Position { x: at.x, y: at.y };
        let start;
        let end;
        if direction == SearchDirection::Forward {
            start = at.y;
            end = self.rows.len();
        } else {
            start = 0;
            end = at.y.saturating_add(1);
        }

        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        None
    }

    #[must_use]
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    #[must_use]
    pub fn file_type(&self) -> String {
        self.filetype.name()
    }

    pub fn highlight(&mut self, query: Option<&str>) {
        let opts = self.filetype.highlighting_opts();
        for row in &mut self.rows {
            row.highlight(query, opts);
        }
    }
}
