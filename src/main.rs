#![warn(clippy::all, clippy::pedantic)]
mod editor;

mod terminal;
pub use terminal::Terminal;

use editor::Editor;
pub use editor::Position;

fn main() {
    let mut editor = Editor::new();
    editor.run();
}
