//! fsd-showcase — Component gallery for FreeSynergy.Desktop.
//!
//! Only meaningful in debug builds. In release mode it exits immediately.

fn main() {
    #[cfg(not(debug_assertions))]
    {
        eprintln!("fsd-showcase is a debug-only tool. Run with `cargo run` (without --release).");
        return;
    }

    #[cfg(debug_assertions)]
    run();
}

#[cfg(debug_assertions)]
fn run() {
    use tracing_subscriber::EnvFilter;

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting fsd-showcase");

    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::Config;

        dioxus::LaunchBuilder::desktop()
            .with_cfg(
                Config::new().with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("FreeSynergy – Component Showcase")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1400.0_f64, 900.0_f64))
                        .with_resizable(true),
                ),
            )
            .launch(showcase_app);
    }
}

#[cfg(debug_assertions)]
#[allow(non_snake_case)]
fn showcase_app() -> dioxus::prelude::Element {
    use dioxus::prelude::*;

    rsx! {
        div {
            style: "font-family: monospace; padding: 32px; background: #0d1117; color: #e6edf3; min-height: 100vh;",
            h1 { style: "color: #00BCD4;", "FreeSynergy — Component Showcase" }
            p  { style: "color: #8b949e;", "Components will be listed here as they are added to fsn-components." }
            // TODO: import from fsn-components once E5 is implemented
        }
    }
}
