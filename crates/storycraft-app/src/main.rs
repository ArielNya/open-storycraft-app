//! Desktop entry for Open Storycraft.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(missing_docs)]
#![allow(clippy::print_stdout)]

fn main() {
    storycraft_app::run();
}
