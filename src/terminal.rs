use std::io::{self, stdout};
use std::io::{stdin, Write};
use termion::color;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use termion::raw::RawTerminal;

use crate::Position;

pub struct Size {
    height: u16,
    width: u16,
}

pub struct Terminal {
    size: Size,
    _stdout: RawTerminal<io::Stdout>,
}

impl Terminal {
    pub fn new() -> Result<Self, io::Error> {
        let size = termion::terminal_size()?;
        Ok(Self {
            size: Size {
                height: size.1.saturating_sub(2),
                width: size.0,
            },
            _stdout: stdout().into_raw_mode()?,
        })
    }

    pub fn width(&self) -> u16 {
        self.size.width
    }

    pub fn height(&self) -> u16 {
        self.size.height
    }

    pub fn clear_screen() {
        print!("{}", termion::clear::All);
    }

    pub fn cursor_position(position: &Position) {
        let Position { mut x, mut y } = position;
        // avoid overflow of x and y
        x = x.saturating_add(1);
        y = y.saturating_add(1);
        print!("{}", termion::cursor::Goto(x as u16, y as u16));
    }

    pub fn flush() -> Result<(), io::Error> {
        stdout().flush()
    }

    pub fn read_key() -> Result<Key, io::Error> {
        loop {
            if let Some(key) = stdin().lock().keys().next() {
                return key;
            }
        }
    }

    pub fn hide_cursor() {
        print!("{}", termion::cursor::Hide);
    }

    pub fn show_cursor() {
        print!("{}", termion::cursor::Show);
    }

    pub fn clear_current_line() {
        print!("{}", termion::clear::CurrentLine);
    }

    pub fn set_bg_color(color: color::Rgb) {
        print!("{}", color::Bg(color));
    }
        
    pub fn reset_bg_color() {
        print!("{}", color::Bg(color::Reset));
    }

    pub fn set_fg_color(color: color::Rgb) {
        print!("{}", color::Fg(color));
    }
        
    pub fn reset_fg_color() {
        print!("{}", color::Fg(color::Reset));
    }
}
