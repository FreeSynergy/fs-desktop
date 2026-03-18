fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::Config;
        dioxus::LaunchBuilder::desktop()
            .with_cfg(
                Config::new().with_window(
                    dioxus::desktop::WindowBuilder::new()
                        .with_title("FSN Lenses")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1000.0_f64, 700.0_f64)),
                ),
            )
            .launch(fsd_lenses::LensesApp);
    }
}
