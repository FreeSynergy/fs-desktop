#![deny(clippy::all, clippy::pedantic, warnings)]
// Standalone launcher binary for fs-gui-workspace (iced).
// The primary launcher is fs-app — this binary exists for development convenience.
//
// G1.5: Desktop starts at a large default size (fullscreen via OS window manager).

fn main() {
    fs_gui_workspace::init_i18n();

    #[cfg(feature = "iced")]
    {
        use fs_gui_engine_iced::IcedEngine;
        use fs_gui_workspace::shell::{DesktopMessage, DesktopShell};
        let _ = IcedEngine::run_app_with_sub::<DesktopShell, DesktopMessage, _, _, _>(
            "FreeSynergy Desktop",
            DesktopShell::update,
            DesktopShell::view,
            DesktopShell::subscription,
        );
    }
}
