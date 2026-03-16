/// Appearance settings — wallpaper, CSS theme, logo, dark/light mode.
use dioxus::prelude::*;

/// Appearance settings component.
///
/// Reads and writes the global `Signal<String>` theme context provided by
/// `Desktop`. Falls back to a local signal when running standalone (e.g. in
/// fsd-settings standalone mode without a Desktop context).
#[component]
pub fn AppearanceSettings() -> Element {
    // Use the shared theme context when available (provided by Desktop).
    let theme_ctx: Option<Signal<String>> = use_context();
    let mut local_theme = use_signal(|| "dark".to_string());

    let mut wallpaper_url = use_signal(String::new);
    let mut css_url = use_signal(String::new);

    // Determine current theme value from context or local fallback.
    let current_theme = theme_ctx
        .as_ref()
        .map(|s| s.read().clone())
        .unwrap_or_else(|| local_theme.read().clone());

    let mut set_theme = move |value: &'static str| {
        if let Some(mut ctx) = theme_ctx {
            ctx.set(value.to_string());
        } else {
            local_theme.set(value.to_string());
        }
    };

    let light_border = if current_theme == "light" { "var(--fsn-color-primary)" } else { "var(--fsn-color-border-default)" };
    let dark_border  = if current_theme == "dark"  { "var(--fsn-color-primary)" } else { "var(--fsn-color-border-default)" };

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
                        style: "flex: 1; padding: 10px; border-radius: var(--fsn-radius-md); \
                                border: 2px solid {light_border}; cursor: pointer; \
                                background: #f8fafc; color: #0f172a;",
                        onclick: move |_| set_theme("light"),
                        "☀ Light"
                    }
                    button {
                        style: "flex: 1; padding: 10px; border-radius: var(--fsn-radius-md); \
                                border: 2px solid {dark_border}; cursor: pointer; \
                                background: #1e293b; color: white;",
                        onclick: move |_| set_theme("dark"),
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
