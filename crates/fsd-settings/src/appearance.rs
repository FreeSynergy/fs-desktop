/// Appearance settings — theme selector, wallpaper, logo.
use dioxus::prelude::*;

/// Appearance settings component.
///
/// Reads and writes the global `Signal<String>` theme context provided by
/// `Desktop`. Falls back to a local signal when running standalone.
/// The wallpaper context (`Signal<String>`, the CSS background value) is also
/// provided by `Desktop` — updates here propagate to the live desktop background.
#[component]
pub fn AppearanceSettings() -> Element {
    let theme_ctx: Option<Signal<String>> = try_use_context();
    let wallpaper_ctx: Option<Signal<String>> = try_use_context();
    let mut local_theme = use_signal(|| "midnight-blue".to_string());

    let mut wallpaper_url = use_signal(String::new);
    let mut css_url = use_signal(String::new);

    let current_theme = theme_ctx
        .as_ref()
        .map(|s| s.read().clone())
        .unwrap_or_else(|| local_theme.read().clone());

    let set_theme = move |value: String| {
        if let Some(mut ctx) = theme_ctx {
            ctx.set(value);
        } else {
            local_theme.set(value);
        }
    };

    // (id, display_name, "bg,primary,text")
    let themes: &[(&str, &str, &str)] = &[
        ("midnight-blue", "Midnight Blue", "#0c1222,#4d8bf5,#e8edf5"),
        ("cloud-white",   "Cloud White",   "#f8fafc,#2563eb,#0f172a"),
        ("cupertino",     "Cupertino",     "#f5f5f7,#007AFF,#1d1d1f"),
        ("nordic",        "Nordic",        "#2E3440,#88C0D0,#ECEFF4"),
        ("rose-pine",     "Rose Pine",     "#191724,#ebbcba,#e0def4"),
    ];

    rsx! {
        div {
            class: "fsd-appearance",
            style: "padding: 24px; max-width: 600px;",

            h3 { style: "margin-top: 0;", "Appearance" }

            // Theme selector
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 8px;", "Color Theme" }
                div { style: "display: grid; grid-template-columns: repeat(5, 1fr); gap: 8px;",
                    for (id, name, colors) in themes.iter().copied() {
                        {
                            let parts: Vec<&str> = colors.split(',').collect();
                            let (bg, primary, text) = (parts[0], parts[1], parts[2]);
                            let active = current_theme == id;
                            let outline = if active {
                                format!("2px solid {primary}")
                            } else {
                                "2px solid transparent".to_string()
                            };
                            let id_owned = id.to_string();
                            let mut set_theme = set_theme.clone();
                            rsx! {
                                button {
                                    key: "{id}",
                                    style: "padding: 0; border-radius: var(--fsn-radius-md); \
                                            border: none; outline: {outline}; cursor: pointer; \
                                            overflow: hidden; display: flex; flex-direction: column;",
                                    onclick: move |_| set_theme(id_owned.clone()),
                                    div {
                                        style: "height: 40px; background: {bg}; \
                                                display: flex; align-items: center; \
                                                justify-content: center; gap: 4px;",
                                        span {
                                            style: "width: 10px; height: 10px; border-radius: 50%; \
                                                    background: {primary};"
                                        }
                                        span {
                                            style: "width: 10px; height: 10px; border-radius: 50%; \
                                                    background: {text}; opacity: 0.6;"
                                        }
                                    }
                                    div {
                                        style: "padding: 4px 6px; font-size: 10px; text-align: center; \
                                                background: var(--fsn-bg-surface); \
                                                color: var(--fsn-text-primary); \
                                                white-space: nowrap; overflow: hidden; \
                                                text-overflow: ellipsis; width: 100%;",
                                        "{name}"
                                    }
                                }
                            }
                        }
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
                        style: "flex: 1; padding: 8px 12px; \
                                border: 1px solid var(--fsn-color-border-default); \
                                border-radius: var(--fsn-radius-md); \
                                background: var(--fsn-color-bg-base); \
                                color: var(--fsn-color-text-primary);",
                        oninput: move |e| *wallpaper_url.write() = e.value(),
                    }
                    button {
                        style: "padding: 8px 12px; background: var(--fsn-color-primary); \
                                color: white; border: none; \
                                border-radius: var(--fsn-radius-md); cursor: pointer;",
                        onclick: move |_| {
                            let url = wallpaper_url.read().clone();
                            if !url.is_empty() {
                                let css = format!(
                                    "background-image: url('{}'); background-size: cover; background-position: center;",
                                    url
                                );
                                if let Some(mut ctx) = wallpaper_ctx {
                                    ctx.set(css);
                                }
                            }
                        },
                        "Load URL"
                    }
                }
                label {
                    style: "display: flex; align-items: center; gap: 8px; \
                            padding: 8px 12px; border: 1px dashed var(--fsn-color-border-default); \
                            border-radius: var(--fsn-radius-md); cursor: pointer;",
                    input {
                        r#type: "file",
                        accept: "image/*",
                        style: "display: none;",
                        onchange: move |e| {
                            if let Some(engine) = e.files() {
                                let files = engine.files();
                                if let Some(path) = files.into_iter().next() {
                                    let css = format!(
                                        "background-image: url('file://{}'); background-size: cover; background-position: center;",
                                        path
                                    );
                                    if let Some(mut ctx) = wallpaper_ctx {
                                        ctx.set(css);
                                    }
                                }
                            }
                        },
                    }
                    "📁 Upload from file"
                }
                // Reset to default
                button {
                    style: "margin-top: 8px; padding: 6px 12px; font-size: 12px; \
                            background: transparent; border: 1px solid var(--fsn-color-border-default); \
                            border-radius: var(--fsn-radius-md); cursor: pointer; \
                            color: var(--fsn-color-text-muted);",
                    onclick: move |_| {
                        if let Some(mut ctx) = wallpaper_ctx {
                            ctx.set("background: linear-gradient(135deg, #0f172a, #1e293b);".to_string());
                        }
                        wallpaper_url.set(String::new());
                    },
                    "Reset to default"
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
                    "Upload theme file"
                }
            }

            // Logo
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 8px;", "Logo" }
                label {
                    style: "display: flex; align-items: center; gap: 8px; padding: 8px 12px; border: 1px dashed var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: pointer;",
                    input { r#type: "file", accept: "image/*,.svg", style: "display: none;" }
                    "Upload logo (SVG or PNG)"
                }
            }

            button {
                style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                "Apply"
            }
        }
    }
}
