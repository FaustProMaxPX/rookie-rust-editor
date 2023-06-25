use std::{
    error::Error,
    io::{self},
};

use termion::event::Key;

use crate::Terminal;

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Editor {
    stop: bool,
    terminal: Terminal,
    position: Position,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            stop: false,
            terminal: Terminal::new().expect("can't create a terminal"),
            position: Position { x: 0, y: 0 },
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

    fn draw_rows(&self) {
        let height = self.terminal.height();

        for row in 0..height - 1 {
            Terminal::clear_current_line();
            if row == height / 3 {
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
                self.stop = true;
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
        Ok(())
    }

    fn move_cursor(&mut self, key: Key) {
        let Position { mut x, mut y } = self.position;
        let height = self.terminal.height().saturating_sub(1) as usize;
        let width = self.terminal.width().saturating_sub(1) as usize;

        match key {
            Key::Up => y = y.saturating_sub(1),
            // avoid moving the cursor off
            Key::Down if y < height => y = y.saturating_add(1),
            Key::Left => x = x.saturating_sub(1),
            Key::Right if x < width => x = x.saturating_add(1),
            Key::PageUp => y = 0,
            Key::PageDown => y = height,
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        }
        self.position = Position { x, y };
    }

    fn refresh_screen(&self) -> Result<(), io::Error> {
        Terminal::hide_cursor();
        Terminal::cursor_position(&Position { x: 0, y: 0 });

        if self.stop {
            Terminal::clear_screen();
            println!("goodbye\r");
        } else {
            self.draw_rows();
            // after draw `~`, we need to put the cursor back
            Terminal::cursor_position(&self.position);
        }

        Terminal::show_cursor();
        Terminal::flush()
    }
}

fn die(e: Box<dyn Error>) {
    Terminal::clear_screen();
    panic!("{}", e);
}
