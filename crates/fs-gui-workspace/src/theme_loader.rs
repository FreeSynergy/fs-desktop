// Theme loader — re-exports from fs-theme.
//
// Store themes are saved WITHOUT a CSS variable prefix (e.g. `--bg-base`).
// Each program adds its own prefix when loading (e.g. `--fs-bg-base` for Desktop).
// See technik/css.md for the full naming convention.

pub use fs_theme::{REQUIRED_VARS, prefix_theme_css, validate_theme_vars};
