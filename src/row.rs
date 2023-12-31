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
        result.push_str(&format!("{}", color::Fg(cur_color_type.to_color())));

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

    pub fn highlight(&mut self, query: Option<&str>, hl_opts: &HighlightingOptions) {
        self.highlighting.clear();
        let chars: Vec<char> = self.content.chars().collect();
        let mut index = 0;
        while chars.get(index).is_some() {
            if !self.highlight_comment(&chars, hl_opts, &mut index)
                && !self.highlight_character(&chars, hl_opts, &mut index)
                && !self.highlight_strings(&chars, hl_opts, &mut index)
                && !self.highlight_number(&chars, hl_opts, &mut index)
                && !self.highlight_primary_keys(&chars, hl_opts, &mut index)
                && !self.highlight_secondary_keys(&chars, hl_opts, &mut index)
            {
                self.highlighting.push(Type::None);
                index += 1;
            }
        }

        self.highlight_match(query);
    }

    fn highlight_target_str(
        &mut self,
        chars: &Vec<char>,
        target: &str,
        index: &mut usize,
        hl_type: Type,
    ) -> bool {
        if *index > 0 && !self.is_separator(chars[*index - 1]) {
            return false;
        }

        if let Some(c) = chars.get(target.len().saturating_add(*index)) {
            if !self.is_separator(*c) {
                return false;
            }
        }

        if target.is_empty() {
            return false;
        }
        for (i, c) in target.chars().enumerate() {
            if let Some(next_char) = chars.get(i.saturating_add(*index)) {
                if *next_char != c {
                    return false;
                }
            } else {
                return false;
            }
        }
        for _ in 0..target.len() {
            self.highlighting.push(hl_type);
            *index += 1;
        }
        true
    }

    fn highlight_primary_keys(
        &mut self,
        chars: &Vec<char>,
        hl_opts: &HighlightingOptions,
        index: &mut usize,
    ) -> bool {
        if hl_opts.primary_keys().is_empty() {
            return false;
        }
        for keyword in hl_opts.primary_keys() {
            if self.highlight_target_str(chars, keyword, index, Type::PrimaryKey) {
                return true;
            }
        }
        false
    }

    fn highlight_secondary_keys(
        &mut self,
        chars: &Vec<char>,
        hl_opts: &HighlightingOptions,
        index: &mut usize,
    ) -> bool {
        if hl_opts.secondary_keys().is_empty() {
            return false;
        }
        for keyword in hl_opts.secondary_keys() {
            if self.highlight_target_str(chars, keyword, index, Type::SecondaryKey) {
                return true;
            }
        }
        false
    }

    fn highlight_number(
        &mut self,
        chars: &Vec<char>,
        hl_opts: &HighlightingOptions,
        index: &mut usize,
    ) -> bool {
        if !hl_opts.numbers() {
            return false;
        }
        let mut prev_is_number = false;
        let mut chganged = false;
        while let Some(c) = chars.get(*index) {
            if c.is_ascii_digit() || (*c == '.' && prev_is_number) {
                prev_is_number = true;
                chganged = true;
                self.highlighting.push(Type::Number);
                *index += 1;
            } else {
                break;
            }
        }
        chganged
    }

    fn highlight_comment(
        &mut self,
        chars: &Vec<char>,
        hl_opts: &HighlightingOptions,
        index: &mut usize,
    ) -> bool {
        if !hl_opts.comments() || chars.get(*index).is_none() {
            return false;
        }

        let c = chars.get(*index).unwrap();
        if let Some(next_char) = chars.get((*index).saturating_add(1)) {
            if *c == '/' && *next_char == '/' {
                let start = *index;
                for _ in start..chars.len() {
                    self.highlighting.push(Type::Comment);
                    *index += 1;
                }
                return true;
            }
        }
        false
    }

    fn highlight_match(&mut self, query: Option<&str>) {
        let mut search_index = 0;

        if let Some(query) = query {
            if query.is_empty() {
                return;
            }
            let query_len = query.graphemes(true).count();
            while let Some(search_match) = self.find(query, search_index, SearchDirection::Forward)
            {
                // add query len to search_index as the next start index
                // we use checked_add here in that it will return None if the result is out of range
                if let Some(next_index) = search_match.checked_add(query_len) {
                    for i in search_match..next_index {
                        self.highlighting[i] = Type::Match;
                    }
                    search_index = next_index;
                } else {
                    break;
                }
            }
        }
    }

    fn highlight_character(
        &mut self,
        chars: &Vec<char>,
        hl_opts: &HighlightingOptions,
        index: &mut usize,
    ) -> bool {
        if !hl_opts.characters() {
            return false;
        }
        if let Some(c) = chars.get(*index) {
            if *c != '\'' {
                return false;
            }
        }
        let start_index = *index;
        if let Some(next_char) = chars.get(*index + 1) {
            if *next_char == '\\' {
                *index += 4;
            } else {
                *index += 3;
            }
            for _ in start_index..*index {
                self.highlighting.push(Type::Character);
            }
            return true;
        }
        false
    }

    fn highlight_strings(
        &mut self,
        chars: &Vec<char>,
        hl_opts: &HighlightingOptions,
        index: &mut usize,
    ) -> bool {
        if !hl_opts.strings() {
            return false;
        }

        // if this character is in string, push a Type::String
        // if current character is '"', it means we have been in the end of string
        if let Some(c) = chars.get(*index) {
            if *c != '"' {
                return false;
            }
        }
        *index += 1;
        self.highlighting.push(Type::String);
        while let Some(ch) = chars.get(*index) {
            if *ch == '"' {
                self.highlighting.push(Type::String);
                *index += 1;
                return true;
            } else if *ch == '\\' {
                self.highlighting.push(Type::Escape);
                self.highlighting.push(Type::Escape);
                *index += 2;
            } else {
                self.highlighting.push(Type::String);
                *index += 1;
            }
        }
        false
    }

    fn is_separator(&self, ch: char) -> bool {
        ch.is_ascii_whitespace() || ch.is_ascii_punctuation()
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

mod row_tests {
    use crate::FileType;

    use super::*;

    #[test]
    fn highlight_strings_test() {
        let mut row = Row::from("\"h\\nello\"");
        let hl_opts = FileType::from("a.rs").highlighting_opts().clone();
        let chars: Vec<char> = row.content.chars().collect();
        let mut index = 0;
        row.highlight_strings(&chars, &hl_opts, &mut index);
        let mut expected = Vec::new();
        for i in 0..9 {
            if i == 2 || i == 3 {
                expected.push(Type::Escape);
                continue;
            }
            expected.push(Type::String);
        }
        assert_eq!(
            row.highlighting, expected,
            "res: {:#?}, expected: {:#?}",
            row.highlighting, expected
        );
    }

    #[test]
    fn highlight_character_test() {
        let (mut row, hl_opts) = create_row("'1'");
        let mut index = 0;
        let chars = row.content.chars().collect();
        row.highlight_character(&chars, &hl_opts, &mut index);
        let mut expected = Vec::new();
        for _i in 0..3 {
            expected.push(Type::Character);
        }
        assert_eq!(
            row.highlighting, expected,
            "res: {:#?}, expected: {:#?}",
            row.highlighting, expected
        );
        assert_eq!(index, 3);

        let (mut row, hl_opts) = create_row("'\\n'");
        let mut index = 0;
        let chs = row.content.chars().collect();
        row.highlight_character(&chs, &hl_opts, &mut index);
        expected.clear();
        for _ in 0..4 {
            expected.push(Type::Character);
        }
        assert_eq!(
            row.highlighting, expected,
            "res: {:#?}, expected: {:#?}",
            row.highlighting, expected
        );
        assert_eq!(index, 4);
    }

    #[test]
    fn highlight_comment_test() {
        let (mut row, hl_opts) = create_row("// this is a comment");
        let mut index = 0;
        row.highlight_comment(&row.content.chars().collect(), &hl_opts, &mut index);
        let mut expected = Vec::new();
        for _i in 0..20 {
            expected.push(Type::Comment);
        }
        assert_eq!(
            row.highlighting, expected,
            "res: {:#?}, expected: {:#?}",
            row.highlighting, expected
        );
        assert_eq!(index, 20);
    }

    #[test]
    fn highlight_match_test() {
        let (mut row, _hl_opts) = create_row("111");
        for _ in 0..3 {
            row.highlighting.push(Type::None);
        }
        row.highlight_match(Some("1"));
        let expected = vec![Type::Match, Type::Match, Type::Match];
        assert_eq!(
            row.highlighting, expected,
            "res: {:#?}, expected: {:#?}",
            row.highlighting, expected
        );
    }

    #[test]
    fn highlight_number_test() {
        let (mut row, hl_opts) = create_row("1");
        let mut index = 0;
        row.highlight_number(&row.content.chars().collect(), &hl_opts, &mut index);
        // let mut expected = Vec::new();

        assert_eq!(
            row.highlighting,
            vec![Type::Number],
            "res: {:#?}, expected: {:#?}",
            row.highlighting,
            vec![Type::Number]
        );
        assert_eq!(index, 1);

        index = 0;
        row.content = "1.0".to_string();
        row.highlighting.clear();
        row.highlight_number(&row.content.chars().collect(), &hl_opts, &mut index);
        assert_eq!(
            row.highlighting,
            vec![Type::Number, Type::Number, Type::Number],
            "res: {:#?}, expected: {:#?}",
            row.highlighting,
            vec![Type::Number, Type::Number, Type::Number]
        );

        index = 0;
        row.content = "\"1.0\"".to_string();
        row.highlighting.clear();
        assert!(!row.highlight_number(&row.content.chars().collect(), &hl_opts, &mut index));
    }

    fn create_row(string: &str) -> (Row, HighlightingOptions) {
        let row = Row::from(string);
        let hl_opts = FileType::from("a.rs").highlighting_opts().clone();
        (row, hl_opts)
    }

    fn render_test() {
        let (mut row, hl_opts) = create_row("1");
        let mut index = 0;
        row.highlight(None, &hl_opts);
        
    }
}
