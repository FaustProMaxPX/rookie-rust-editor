use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    content: String,
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
        for g in self.content[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
        {
            if g == "\t" {
                result.push(' ');
            } else {
                result.push_str(g);
            }
        }
        result
    }

    pub fn append(&mut self, row: &Row) {
        self.content = format!("{}{}", self.content, row.content);
        self.len += row.len;
    }
}

impl From<&str> for Row {
    fn from(value: &str) -> Self {
        Self {
            content: String::from(value),
            len: value.graphemes(true).count(),
        }
    }
}

impl Row {
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
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.content.as_bytes()
    }

    pub fn find(&self, query: &str) -> Option<usize> {
        // find the first occurence by string api
        let match_index = self.content.find(query);
        // but the byte index may not equal to grapheme index
        // so we should convert it
        if let Some(match_index) = match_index {
            for (g, (byte_index, _)) in self.content[..].grapheme_indices(true).enumerate() {
                if byte_index == match_index {
                    return Some(g);
                }
            }
        }
        None
    }
}
