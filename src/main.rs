#![warn(clippy::all, clippy::pedantic, clippy::restriction)]
#![allow(
    clippy::missing_docs_in_private_items,
    clippy::implicit_return,
    clippy::shadow_reuse,
    clippy::print_stdout,
    clippy::wildcard_enum_match_arm,
    clippy::else_if_without_else,
)]
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
