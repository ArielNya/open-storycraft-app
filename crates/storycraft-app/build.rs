//! Embeds `tauri.conf.json` into the binary.

fn main() {
    // The Android build also embeds the skill pack, so a pack edit has to
    // rebuild the library (see `src/pack_install.rs`).
    println!("cargo:rerun-if-changed=../../open-storycraft");
    tauri_build::build();
}
