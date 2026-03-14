/// Appearance settings — wallpaper, CSS theme, logo, dark/light mode.
use dioxus::prelude::*;

/// Appearance settings component.
#[component]
pub fn AppearanceSettings() -> Element {
    let wallpaper_url = use_signal(String::new);
    let css_url = use_signal(String::new);
    let dark_mode = use_signal(|| false);

    rsx! {
        div {
            class: "fsd-appearance",
            style: "padding: 24px; max-width: 600px;",

            h3 { style: "margin-top: 0;", "Appearance" }

            // Dark / Light mode
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 8px;", "Color Scheme" }
                div { style: "display: flex; gap: 8px;",
                    button {
                        style: "flex: 1; padding: 10px; border-radius: var(--fsn-radius-md); border: 2px solid {if !*dark_mode.read() { \"var(--fsn-color-primary)\" } else { \"var(--fsn-color-border-default)\" }}; cursor: pointer; background: #f8fafc;",
                        onclick: move |_| *dark_mode.write() = false,
                        "☀ Light"
                    }
                    button {
                        style: "flex: 1; padding: 10px; border-radius: var(--fsn-radius-md); border: 2px solid {if *dark_mode.read() { \"var(--fsn-color-primary)\" } else { \"var(--fsn-color-border-default)\" }}; cursor: pointer; background: #1e293b; color: white;",
                        onclick: move |_| *dark_mode.write() = true,
                        "☾ Dark"
                    }
                }
            }

            // Wallpaper
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 8px;", "Wallpaper" }
                div { style: "display: flex; gap: 8px; margin-bottom: 8px;",
                    input {
                        r#type: "url",
                        placeholder: "https://example.com/wallpaper.jpg",
                        style: "flex: 1; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);",
                        oninput: move |e| *wallpaper_url.write() = e.value(),
                    }
                    button {
                        style: "padding: 8px 12px; background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: pointer;",
                        "Load URL"
                    }
                }
                label {
                    style: "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border: 1px dashed var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: pointer;",
                    input { r#type: "file", accept: "image/*", style: "display: none;" }
                    "📁 Upload from file"
                }
            }

            // CSS Theme
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 8px;",
                    "CSS Theme (theme.css)"
                }
                p { style: "font-size: 12px; color: var(--fsn-color-text-muted); margin-bottom: 8px;",
                    "Provide a CSS file with --fsn-* custom properties. It will be converted to theme.toml automatically."
                }
                div { style: "display: flex; gap: 8px; margin-bottom: 8px;",
                    input {
                        r#type: "url",
                        placeholder: "https://example.com/theme.css",
                        style: "flex: 1; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);",
                        oninput: move |e| *css_url.write() = e.value(),
                    }
                    button {
                        style: "padding: 8px 12px; background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: pointer;",
                        "Load URL"
                    }
                }
                label {
                    style: "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border: 1px dashed var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: pointer;",
                    input { r#type: "file", accept: ".css,.toml", style: "display: none;" }
                    "📁 Upload theme file"
                }
            }

            // Logo
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 8px;", "Logo" }
                label {
                    style: "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border: 1px dashed var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: pointer;",
                    input { r#type: "file", accept: "image/*,.svg", style: "display: none;" }
                    "📁 Upload logo (SVG or PNG)"
                }
            }

            button {
                style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                "Apply"
            }
        }
    }
}
