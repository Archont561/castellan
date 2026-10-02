//! The `desktop` binary entry point.
//!
//! Everything lives in the `castellan_desktop_lib` library rather than here,
//! because the mobile targets never call `main` -- they enter through the
//! library's `run()` via Tauri's generated entry point. Keeping this file a single
//! delegation is what stops the two doors from drifting apart.

// Prevents an extra console window on Windows in release builds.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    castellan_desktop_lib::run()
}
