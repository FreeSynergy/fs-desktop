#![deny(clippy::all, clippy::pedantic, warnings)]
// FreeSynergy.Desktop — main launcher.
// Engine selection via feature flags (default: iced).

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    std::panic::set_hook(Box::new(|info| {
        tracing::error!("PANIC: {info}");
    }));

    tracing::info!("Starting FreeSynergy.Desktop");

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

    #[cfg(all(feature = "bevy", not(feature = "iced")))]
    {
        tracing::info!("Bevy engine selected — desktop shell not yet implemented");
    }
}
