fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::Config;
        dioxus::LaunchBuilder::desktop()
            .with_cfg(
                Config::new()
                    .with_window(
                        dioxus::desktop::WindowBuilder::new()
                            .with_title("FSN Browser")
                            .with_inner_size(dioxus::desktop::LogicalSize::new(1100.0_f64, 750.0_f64)),
                    )
                    // Allow all navigation so external URLs stay in the WebView
                    // instead of being opened in the system browser.
                    // Tracking PR: https://github.com/DioxusLabs/dioxus/pull/5390
                    .with_navigation_handler(|_url: String| true),
            )
            .launch(fsd_browser::BrowserApp);
    }
}
