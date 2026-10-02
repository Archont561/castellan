//! Build-time half of the Tauri app.
//!
//! `tauri_build::build()` reads `tauri.conf.json` and the files under
//! `capabilities/`, generates the permission scaffolding that
//! `tauri::generate_context!()` expands against in `lib.rs`, and emits the
//! `cargo:rerun-if-changed` lines that make a config, capability or icon
//! edit rebuild the crate instead of baking a stale context into the binary.

fn main() {
    tauri_build::build()
}
