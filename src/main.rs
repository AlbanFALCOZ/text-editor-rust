#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division,
    clippy::indexing_slicing
)]

use crate::editor::Editor;

pub mod editor;

fn main() {
    Editor::new().unwrap().run();
}
