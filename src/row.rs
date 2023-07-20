use termion::color;
use unicode_segmentation::UnicodeSegmentation;

use crate::{highlighting::Type, HighlightingOptions, SearchDirection};

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
        // keep track of current color type
        // then we don't need to change color whenever ecounter a new character if it is the same
        let mut cur_color_type = &Type::None;

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

                // if encounter a new color type, then we need to change the color
                if cur_color_type != htype {
                    cur_color_type = htype;
                    let start_highlighting = format!("{}", color::Fg(htype.to_color()));
                    result.push_str(&start_highlighting);
                }
                if c == '\t' {
                    result.push(' ');
                } else {
                    result.push(c);
                }
            }
        }
        let end_highlighting = format!("{}", color::Fg(color::Reset));
        result.push_str(&end_highlighting);
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

    /// Split the content into two parts
    /// since it will create a new value, so its return value must be used
    #[must_use]
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

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.content.as_bytes()
    }

    #[must_use]
    pub fn find(&self, query: &str, at: usize, direction: SearchDirection) -> Option<usize> {
        if at > self.len || query.is_empty() {
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

    pub fn highlight(&mut self, query: Option<&str>, hl_opts: HighlightingOptions) {
        let mut highlighting = vec![];

        let chars: Vec<char> = self.content.chars().collect();
        let mut matches = vec![];
        let mut search_index = 0;

        if let Some(query) = query {
            let query_len = query.graphemes(true).count();
            while let Some(search_match) = self.find(query, search_index, SearchDirection::Forward)
            {
                matches.push(search_match);
                // add query len to search_index as the next start index
                // we use checked_add here in that it will return None if the result is out of range
                if let Some(next_index) = search_match.checked_add(query_len) {
                    search_index = next_index;
                } else {
                    break;
                }
            }
        }

        let mut prev_is_separator = true;
        // index is the cursor of chars
        let mut index = 0;
        let mut in_string = false;

        while let Some(c) = chars.get(index) {
            if let Some(query) = query {
                if matches.contains(&index) {
                    for _ in query[..].graphemes(true) {
                        index += 1;
                        highlighting.push(Type::Match);
                    }
                    continue;
                }
            }

            let prev_highlighting = if index > 0 {
                highlighting.get(index - 1).unwrap_or(&Type::None)
            } else {
                &Type::None
            };

            if hl_opts.characters() && !in_string && *c == '\'' {
                if let Some(next_char) = chars.get(index + 1) {
                    let closing_index = if *next_char == '\\' {
                        index + 3
                    } else {
                        index + 2
                    };
                    if let Some(closing_char) = chars.get(closing_index) {
                        if *closing_char == '\'' {
                            for _ in 0..=closing_index.saturating_sub(index) {
                                highlighting.push(Type::Character);
                            }
                            index = closing_index;
                        }
                    }
                }
                index += 1;
                highlighting.push(Type::None);
                continue;
            }

            if hl_opts.strings() {

                if *c == '\\' && index < self.len().saturating_sub(1) {
                    highlighting.push(Type::Escape);
                    highlighting.push(Type::Escape);
                    index += 2;
                    continue;
                }

                // if this character is in string, push a Type::String
                // if current character is '"', it means we have been in the end of string
                if in_string {
                    highlighting.push(Type::String);
                    if *c == '"' {
                        in_string = false;
                        prev_is_separator = true;
                    } else {
                        prev_is_separator = false;
                    }
                    index += 1;
                    // the `continue` is very important
                    // that means the mutable reference of highlighting here will not be conflicted with previous_highlighting above
                    continue;
                } else if prev_is_separator && *c == '"' {
                    highlighting.push(Type::String);
                    in_string = true;
                    prev_is_separator = true;
                    index += 1;
                    continue;
                }
            }

            // comments should be put after strings
            // because there may be a '/' in a string
            // it should not be treated as a comment
            if hl_opts.comments() && *c == '/' {
                if let Some(next_char) = chars.get(index + 1) {
                    if *next_char == '/' {
                        for _ in index..chars.len() {
                            highlighting.push(Type::Comment);
                        }
                    }
                    break;
                }
            }

            // TODO: if a character follows the number, the number will still be defined as Type::Number
            if hl_opts.numbers() {
                if (c.is_ascii_digit() && (prev_is_separator || *prev_highlighting == Type::Number))
                    || (*c == '.' && *prev_highlighting == Type::Number)
                {
                    highlighting.push(Type::Number);
                } else {
                    highlighting.push(Type::None);
                }
            } else {
                highlighting.push(Type::None);
            }
            prev_is_separator = c.is_ascii_punctuation() || c.is_ascii_whitespace();
            index += 1;

            
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
