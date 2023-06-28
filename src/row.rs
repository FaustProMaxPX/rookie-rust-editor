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
        self.update_len();
    }
}

impl From<&str> for Row {
    fn from(value: &str) -> Self {
        let mut row = Self {
            content: String::from(value),
            len: 0,
        };
        row.update_len();
        row
    }
}

impl Row {
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    fn update_len(&mut self) {
        self.len = self.content[..].graphemes(true).count();
    }

    pub fn insert(&mut self, at: usize, c: char) {
        // if we need to append a char, just invoke `String::push`
        if at >= self.len() {
            self.content.push(c);
        } else {
            // if we need to insert a char into the middle of content
            // split the content at the position `at`
            // and then push the char to the end of front part
            // the combine the new string with another part
            let mut res: String = self.content[..].graphemes(true).take(at).collect();
            let remainder: String = self.content[..].graphemes(true).skip(at).collect();
            res.push(c);
            res.push_str(&remainder);
            self.content = res;
        }
        self.update_len();
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        }
        let mut res: String = self.content[..].graphemes(true).take(at).collect();
        let remain: String = self.content[..].graphemes(true).skip(at + 1).collect();
        res.push_str(&remain);
        self.content = res;
        self.update_len();
    }

    pub fn split(&mut self, at: usize) -> Self {
        let begin = self.content[..].graphemes(true).take(at).collect();
        let remain: String = self.content[..].graphemes(true).skip(at).collect();
        self.content = begin;
        self.update_len();
        Self::from(&remain[..])
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.content.as_bytes()
    }
}
