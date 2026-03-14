use tracing_subscriber::EnvFilter;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting FreeSynergy.Desktop");

    // Launch the desktop shell (Dioxus handles the event loop)
    #[cfg(feature = "desktop")]
    dioxus::launch(fsd_shell::Desktop);

    #[cfg(feature = "web")]
    dioxus::launch(fsd_shell::Desktop);
}
