use unicode_segmentation::UnicodeSegmentation;

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
}
