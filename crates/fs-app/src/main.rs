// Replace the system allocator to avoid WebKitGTK heap-corruption on window close.
// See: https://github.com/DioxusLabs/dioxus/issues (free(): corrupted unsorted chunks)
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use tracing_subscriber::EnvFilter;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    std::panic::set_hook(Box::new(|info| {
        tracing::error!("PANIC: {info}");
        // TODO: surface via NotificationBus once available
    }));

    tracing::info!("Starting FreeSynergy.Desktop");

    fs_gui_workspace::init_i18n();

    #[cfg(feature = "desktop")]
    fs_gui_workspace::launch_desktop(
        fs_gui_workspace::DesktopConfig::new()
            .with_title("FreeSynergy.Desktop")
            .with_size(1280.0, 800.0)
            .with_min_size(900.0, 600.0)
            .without_decorations()
            .with_background(12, 18, 34, 255)
            .with_all_navigation(),
        fs_gui_workspace::Desktop,
    );

    #[cfg(feature = "web")]
    dioxus::launch(fs_gui_workspace::WebDesktop);
}
