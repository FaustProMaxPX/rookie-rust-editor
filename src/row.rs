use termion::color;
use unicode_segmentation::UnicodeSegmentation;

use crate::{highlighting::Type, SearchDirection};

#[derive(Default)]
pub struct Row {
    content: String,
    // highlighting is controlled by document, `row` just save them
    highlighting: Vec<Type>,
    // avoid repeating calculate the length
    len: usize,
}

impl Row {
    pub fn render(&self, start: usize, end: usize) -> String {
        // we should make sure that end is not greater than len of content
        let end = end.min(self.content.len());
        let start = start.min(end);

        // use a library to deal with the length of unicode string
        let mut result = String::new();
        for (index, g) in self.content[..]
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            if let Some(c) = g.chars().next() {
                // because `highlighting()` is invoked whenever one row is pushed into `rows`
                // we can find coresponding highlighting type by index
                let htype = self.highlighting.get(index).unwrap_or(&Type::None);
                let start_highlighting = format!("{}", color::Fg(htype.to_color()));
                result.push_str(&start_highlighting);
                if c == '\t' {
                    result.push(' ');
                } else {
                    result.push(c);
                }
                let end_highlighting = format!("{}", color::Fg(color::Reset));
                result.push_str(&end_highlighting);

            }
        }
        result
    }

    pub fn append(&mut self, row: &Row) {
        self.content = format!("{}{}", self.content, row.content);
        self.len += row.len;
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn insert(&mut self, at: usize, c: char) {
        // if we need to append a char, just invoke `String::push`
        if at >= self.len() {
            self.content.push(c);
            self.len += 1;
            return;
        }
        // if we need to insert a char into the middle of content
        // split the content at the position `at`
        // and then push the char to the end of front part
        // the combine the new string with another part
        let mut result = String::new();
        let mut length = 0;
        for (index, grapheme) in self.content[..].graphemes(true).enumerate() {
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }
        self.content = result;
        self.len = length;
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        }
        let mut length = 0;
        let mut result = String::new();
        for (index, g) in self.content[..].graphemes(true).enumerate() {
            if index != at {
                length += 1;
                result.push_str(g);
            }
        }
        self.content = result;
        self.len = length;
    }

    pub fn split(&mut self, at: usize) -> Self {
        let mut row = String::new();
        let mut length = 0;
        let mut split_res = String::new();
        let mut split_len = 0;

        for (index, g) in self.content[..].graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(g);
            } else {
                split_len += 1;
                split_res.push_str(g);
            }
        }
        self.content = row;
        self.len = length;
        Self {
            content: split_res,
            len: split_len,
            highlighting: Vec::new(),
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.content.as_bytes()
    }

    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len {
            return None;
        }

        let start;
        let end;
        if direction == SearchDirection::Forward {
            start = at;
            end = self.len;
        } else {
            start = 0;
            end = at;
        }

        let content: String = self.content[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();

        // find the first occurence by string api, match_index is the index of target's first byte
        let match_index = if direction == SearchDirection::Forward {
            content.find(query)
        } else {
            content.rfind(query)
        };

        // TODO: why not search the whole string with a single loop. Current solution iterates through the string for 3 times
        if let Some(match_index) = match_index {
            // we should find the unicode byte corresponding to that byteindex
            for (g, (byte_index, _)) in content[..].grapheme_indices(true).enumerate() {
                if match_index == byte_index {
                    return Some(start + g);
                }
            }
        }
        None
    }

    pub fn highlight(&mut self) {
        let mut highlighting = vec![];
        for c in self.content.chars() {
            if c.is_ascii_digit() {
                highlighting.push(Type::Number);
            } else {
                highlighting.push(Type::None);
            }
        }
        self.highlighting = highlighting;
    }
}

impl From<&str> for Row {
    fn from(value: &str) -> Self {
        Self {
            content: String::from(value),
            highlighting: Vec::new(),
            len: value.graphemes(true).count(),
        }
    }
}
