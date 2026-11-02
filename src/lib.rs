//! Search Iconify icon sets from the terminal.
//!
//! The binary in `main.rs` is a thin layer over these modules: the API client,
//! icon names, license classes, the Japanese keyword table, rendering, and the
//! HTML and credits reports.

pub mod api;
pub mod credits;
pub mod html;
pub mod id;
pub mod ja;
pub mod license;
pub mod render;
