#![warn(clippy::all, clippy::pedantic)]
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
mod highlighting;
pub use terminal::Terminal;

mod document;
pub use document::Document;

mod row;
pub use row::Row;

use editor::Editor;
pub use editor::Position;
pub use editor::SearchDirection;

fn main() {
    let mut editor = Editor::new();
    editor.run();
}
