//! Snapshot tests for fs-shell CSS/config outputs.
//! Uses `insta` for deterministic snapshot diffs.
//!
//! Run `cargo insta review` to accept new/changed snapshots.

#[test]
fn theme_css_vars_snapshot() {
    let css = fs_theme::ThemeEngine::default().to_css();
    insta::assert_snapshot!(css);
}

#[test]
fn theme_full_css_snapshot() {
    let css = fs_theme::ThemeEngine::default().to_full_css();
    insta::assert_snapshot!(css);
}

#[test]
fn theme_glass_css_snapshot() {
    let css = fs_theme::ThemeEngine::glass_css();
    insta::assert_snapshot!(css);
}

#[test]
fn theme_animations_css_snapshot() {
    let css = fs_theme::ThemeEngine::animations_css();
    insta::assert_snapshot!(css);
}
