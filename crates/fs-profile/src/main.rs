#![deny(clippy::all, clippy::pedantic, warnings)]
fn main() {
    #[cfg(feature = "iced")]
    {
        use fs_gui_engine_iced::IcedEngine;
        use fs_profile::app::{ProfileApp, ProfileMessage};
        let _ = IcedEngine::run_app::<ProfileApp, ProfileMessage, _, _>(
            "FreeSynergy — Profile",
            ProfileApp::update,
            ProfileApp::view,
        );
    }
    #[cfg(not(feature = "iced"))]
    {
        eprintln!("fs-profile: no GUI engine enabled");
    }
}
