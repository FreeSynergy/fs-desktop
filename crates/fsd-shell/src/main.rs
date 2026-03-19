fn main() {
    #[cfg(feature = "desktop")]
    dioxus::LaunchBuilder::desktop()
        .with_cfg(
            dioxus::desktop::Config::new()
                // Allow all navigation within the WebView so the Browser app
                // can load external URLs in iframes instead of the system browser.
                // Tracking PR: https://github.com/DioxusLabs/dioxus/pull/5390
                .with_navigation_handler(|_url: String| true),
        )
        .launch(fsd_shell::Desktop);
}
