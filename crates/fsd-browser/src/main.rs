fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::Config;
        dioxus::LaunchBuilder::desktop()
            .with_cfg(
                Config::new().with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("FSN Browser")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1100.0_f64, 750.0_f64)),
                ),
            )
            .launch(fsd_browser::BrowserApp);
    }
}
