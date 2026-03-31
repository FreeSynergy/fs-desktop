#![deny(clippy::all, clippy::pedantic, warnings)]
// Standalone launcher binary for fs-gui-workspace (iced).
// The primary launcher is fs-app — this binary exists for development convenience.

fn main() {
    fs_gui_workspace::init_i18n();

    #[cfg(feature = "iced")]
    {
        use fs_gui_engine_iced::IcedEngine;
        use fs_gui_workspace::shell::{DesktopMessage, DesktopShell};
        let _ = IcedEngine::run_app::<DesktopShell, DesktopMessage, _, _>(
            "FreeSynergy Desktop",
            DesktopShell::update,
            DesktopShell::view,
        );
    }
}
