use std::{
    env,
    error::Error,
    io,
    time::{Duration, Instant},
};

use termion::{color, event::Key};

use crate::{row, Document, Terminal};

const VERSION: &str = env!("CARGO_PKG_VERSION");

const QUIT_TIMES: u8 = 1;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(text: String) -> Self {
        Self {
            text,
            time: Instant::now(),
        }
    }
}

impl From<&str> for StatusMessage {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

pub struct Editor {
    stop: bool,
    terminal: Terminal,
    // cusor position
    position: Position,
    document: Document,
    // keep track of what row of the file the user is currently scrolled to
    offset: Position,
    status_message: StatusMessage,
    // confirmation of quit when the user presses CTRL-Q and there are some changes have not been saved
    quit_times: u8,
}

impl Editor {
    pub fn new() -> Self {
        let args: Vec<String> = env::args().collect();
        let mut init_status = String::from("HELP: CTRL-Q quit | CTRL-S save | CTRL-F search");
        let document = if let Some(filename) = args.get(1) {
            if let Ok(doc) = Document::open(filename) {
                doc
            } else {
                init_status = format!("ERR: Cannot open file: {filename}");
                Document::default()
            }
        } else {
            Document::default()
        };
        Self {
            stop: false,
            terminal: Terminal::new().expect("can't create a terminal"),
            position: Position::default(),
            document,
            offset: Position::default(),
            status_message: StatusMessage::from(init_status),
            quit_times: QUIT_TIMES,
        }
    }

    pub fn run(&mut self) {
        loop {
            if let Err(e) = self.refresh_screen() {
                die(Box::new(e));
            }

            if self.stop {
                break;
            }

            if let Err(e) = self.process_key() {
                die(Box::new(e));
            }
        }
    }

    pub fn draw_row(&self, row: &row::Row) {
        let width = self.terminal.width() as usize;
        let start = self.offset.x;
        let end = self.offset.x.saturating_add(width);
        let row = row.render(start, end);
        println!("{row}\r");
    }

    fn draw_rows(&self) {
        let height = self.terminal.height();

        for terminal_row in 0..height {
            Terminal::clear_current_line();
            // if there are some contents in current row, render it
            // if not, just render the ~ or welcome message
            if let Some(row) = self
                .document
                .row(self.offset.y.saturating_add(terminal_row as usize))
            {
                self.draw_row(row);
            } else if terminal_row == height / 3 && self.document.is_empty() {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_welcome_message(&self) {
        let mut msg = format!("Editor --version {VERSION}\r");
        let width = self.terminal.width() as usize;
        let len = msg.len();

        // center the welcome message
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        msg = format!("~{spaces}{msg}");

        // avoid our message being cutoff
        msg.truncate(width);
        println!("{msg}\r");
    }

    fn process_key(&mut self) -> Result<(), io::Error> {
        let key = Terminal::read_key()?;
        match key {
            Key::Ctrl('q') => {
                if self.quit_times > 0 && self.document.is_dirty() {
                    self.status_message = StatusMessage::from(
                        "WARNING! File has unsaved changes. Press CTRL-Q again to quit."
                            .to_string(),
                    );
                    self.quit_times -= 1;
                    return Ok(());
                }
                self.stop = true;
            }
            Key::Ctrl('s') => self.save(),
            Key::Ctrl('f') => self.search(),
            Key::Char(c) => {
                self.document.insert(&self.position, c);
                self.move_cursor(Key::Right);
            }
            Key::Delete => self.document.delete(&self.position),
            Key::Backspace => {
                if self.position.x > 0 || self.position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.position);
                }
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::Home
            | Key::End => {
                self.move_cursor(key);
            }
            _ => {
                println!("{key:?}\r");
            }
        }
        self.scroll();
        // if this code can be executed, that means user doesn't choose to quit
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = "".into();
        }
        Ok(())
    }

    /// Prompt the user for input. `f` is a closure that will be executed when the user presses a key
    fn prompt<F>(&mut self, prompt: &str, f: F) -> Result<Option<String>, io::Error>
    where
        F: Fn(&mut Self, Key, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{prompt}{result}"));
            self.refresh_screen()?;
            let key = Terminal::read_key()?;
            match key {
                Key::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len() - 1);
                    }
                }
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
            f(self, key, &result);
        }
        self.status_message = "".into();
        if result.is_empty() {
            return Ok(None);
        }
        Ok(Some(result))
    }

    fn save(&mut self) {
        if self.document.filename.is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = "Save aborted".into();
                return;
            }
            self.document.filename = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = "File saved successfully".into();
        } else {
            self.status_message = "Err: writing file failed".into();
        }
    }

    fn search(&mut self) {
        // save the old position so that we can go back to original position if user cancels searching
        let old_position = self.position.clone();
        // incremental search
        if let Some(query) = self
            .prompt("Search: ", |editor, _, query| {
                if let Some(position) = editor.document.find(query) {
                    editor.position = position;
                    editor.scroll();
                }
            })
            .unwrap_or(None)
        {
            if let Some(position) = self.document.find(&query[..]) {
                self.position = position;
            } else {
                self.status_message = format!("{query} is not found").as_str().into();
            }
        } else {
            self.position = old_position;
            self.scroll();
        }
    }

    fn move_cursor(&mut self, key: Key) {
        let terminal_height = self.terminal.height() as usize;
        let Position { mut x, mut y } = self.position;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        match key {
            Key::Up => y = y.saturating_sub(1),
            // avoid moving the cursor off
            Key::Down if y < height => y = y.saturating_add(1),
            // if the cursor is at the first of current line
            // then we need to move it to the line above if y > 0
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    if let Some(row) = self.document.row(y) {
                        x = row.len();
                    } else {
                        x = 0;
                    }
                }
            }
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            Key::PageUp => {
                y = if y > terminal_height {
                    y.saturating_sub(terminal_height)
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y.saturating_add(terminal_height)
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }

        // since width may be modified, so we need to recalculate it
        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        if x > width {
            x = width;
        }

        self.position = Position { x, y };
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.position;
        let width = self.terminal.width() as usize;
        let height = self.terminal.height() as usize;
        let mut offset = &mut self.offset;

        // modify offset when cursor move out of the screen
        // make sure the cursor always in the screen

        // if cursor move to the upper side of the screen
        // set offset.y to the height of cursor
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            // if cursor move to the lower side of the screen
            // set offset.y to y - height + 1
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        // same as above
        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn refresh_screen(&self) -> Result<(), io::Error> {
        Terminal::hide_cursor();
        Terminal::cursor_position(&Position::default());

        if self.stop {
            Terminal::clear_screen();
            println!("goodbye\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            // after draw `~`, we need to put the cursor back
            Terminal::cursor_position(&Position {
                x: self.position.x.saturating_sub(self.offset.x),
                y: self.position.y.saturating_sub(self.offset.y),
            });
        }

        Terminal::show_cursor();
        Terminal::flush()
    }

    fn draw_status_bar(&self) {
        let mut status;
        let width = self.terminal.width() as usize;

        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };

        let mut filename = "[No Name]".to_string();
        if let Some(name) = &self.document.filename {
            filename = name.clone();
            filename.truncate(20);
        }

        status = format!(
            "{} - {} lines{}",
            filename,
            self.document.len(),
            modified_indicator
        );

        let line_indicator = format!(
            "{}/{}",
            self.position.y.saturating_add(1),
            self.document.len()
        );
        #[allow(clippy::integer_arithmetic)]
        let len = status.len() + line_indicator.len();

        // fill the status bar if its content is shorter than screen
        if width > len {
            status.push_str(&" ".repeat(width.saturating_sub(len)));
        }

        status = format!("{status}{line_indicator}");
        status.truncate(width);

        Terminal::set_bg_color(STATUS_BG_COLOR);

        Terminal::set_fg_color(STATUS_FG_COLOR);
        println!("{status}\r");

        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let msg = &self.status_message;
        // editor will show status message only for 5 seconds
        if msg.time.elapsed() < Duration::from_secs(5) {
            // we must clone it because we want to truncate the text
            // when the message is showed on screen
            // but we don't want to change the text in the data structure
            let mut text = msg.text.clone();
            text.truncate(self.terminal.width() as usize);
            print!("{text}\r");
        }
    }
}

fn die(e: Box<dyn Error>) {
    Terminal::clear_screen();
    panic!("{}", e);
}
