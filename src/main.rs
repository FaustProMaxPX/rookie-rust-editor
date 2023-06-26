#![warn(clippy::all, clippy::pedantic)]
mod editor;

mod terminal;
pub use terminal::Terminal;

mod document;
pub use document::Document;

mod row;
pub use row::Row;

use editor::Editor;
pub use editor::Position;

fn main() {
    let mut editor = Editor::new();
    editor.run();
}
