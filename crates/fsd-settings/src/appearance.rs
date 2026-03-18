/// Appearance settings — theme selector, wallpaper, theme editor, animation/chrome toggles.
use dioxus::prelude::*;
use fsn_i18n;

/// Appearance settings component.
///
/// Reads and writes the global `Signal<String>` theme context provided by `Desktop`.
/// Falls back to a local signal when running standalone.
#[component]
pub fn AppearanceSettings() -> Element {
    use fsd_db::package_registry::PackageRegistry;
    use fsn_theme::{prefix_theme_css, validate_theme_vars};
    let theme_ctx: Option<Signal<String>> = try_use_context();
    let wallpaper_ctx: Option<Signal<String>> = try_use_context();
    let mut local_theme = use_signal(|| "midnight-blue".to_string());

    let mut wallpaper_url = use_signal(String::new);

    // Theme editor state
    let mut custom_css    = use_signal(String::new);
    let mut editor_error  = use_signal(|| Option::<String>::None);
    let mut editor_saved  = use_signal(|| false);

    // Store-installed themes
    let mut store_themes = use_signal(|| PackageRegistry::by_kind("theme"));
    let mut theme_remove_confirm: Signal<Option<String>> = use_signal(|| None);

    // Animation + Window-Chrome toggles (B5)
    let anim_ctx: Option<Signal<bool>>    = try_use_context();
    let chrome_ctx: Option<Signal<f64>>   = try_use_context();
    let chrome_style_ctx: Option<Signal<String>>  = try_use_context();
    let btn_style_ctx: Option<Signal<String>>     = try_use_context();
    let sidebar_style_ctx: Option<Signal<String>> = try_use_context();
    let mut local_anim          = use_signal(|| true);
    let mut local_opacity       = use_signal(|| 0.80f64);
    let mut local_chrome_style  = use_signal(|| "kde".to_string());
    let mut local_btn_style     = use_signal(|| "rounded".to_string());
    let mut local_sidebar_style = use_signal(|| "solid".to_string());

    let current_theme = theme_ctx
        .as_ref()
        .map(|s| s.read().clone())
        .unwrap_or_else(|| local_theme.read().clone());

    let anim_enabled = anim_ctx
        .as_ref()
        .map(|s| *s.read())
        .unwrap_or_else(|| *local_anim.read());

    let chrome_opacity = chrome_ctx
        .as_ref()
        .map(|s| *s.read())
        .unwrap_or_else(|| *local_opacity.read());
    let opacity_pct = (chrome_opacity * 100.0) as u32;
    let opacity_label = if anim_enabled { "On" } else { "Off" };

    let chrome_style = chrome_style_ctx
        .as_ref()
        .map(|s| s.read().clone())
        .unwrap_or_else(|| local_chrome_style.read().clone());
    let btn_style = btn_style_ctx
        .as_ref()
        .map(|s| s.read().clone())
        .unwrap_or_else(|| local_btn_style.read().clone());
    let sidebar_style = sidebar_style_ctx
        .as_ref()
        .map(|s| s.read().clone())
        .unwrap_or_else(|| local_sidebar_style.read().clone());

    let set_theme = move |value: String| {
        if let Some(mut ctx) = theme_ctx {
            ctx.set(value);
        } else {
            local_theme.set(value);
        }
    };

    let mut set_anim = move |enabled: bool| {
        if let Some(mut ctx) = anim_ctx {
            ctx.set(enabled);
        } else {
            local_anim.set(enabled);
        }
    };

    let mut set_opacity = move |v: f64| {
        if let Some(mut ctx) = chrome_ctx {
            ctx.set(v);
        } else {
            local_opacity.set(v);
        }
    };

    let mut set_chrome_style = move |v: String| {
        if let Some(mut ctx) = chrome_style_ctx { ctx.set(v); } else { local_chrome_style.set(v); }
    };
    let mut set_btn_style = move |v: String| {
        if let Some(mut ctx) = btn_style_ctx { ctx.set(v); } else { local_btn_style.set(v); }
    };
    let mut set_sidebar_style = move |v: String| {
        if let Some(mut ctx) = sidebar_style_ctx { ctx.set(v); } else { local_sidebar_style.set(v); }
    };

    // Built-in themes: (id, label, preview colors: bg, primary, text)
    // Only Midnight Blue is bundled. Other themes come from the Store (see "Installed Themes").
    let themes: &[(&str, &str, &str)] = &[
        ("midnight-blue", "Midnight Blue", "#0c1222,#4d8bf5,#e8edf5"),
    ];

    rsx! {
        div {
            class: "fsd-appearance fsn-scrollable",
            style: "padding: 24px; max-width: 680px; height: 100%;",

            // ── Section: Color Theme ───────────────────────────────────────
            h3 { style: "margin-top: 0; margin-bottom: 16px; font-size: 16px;", {fsn_i18n::t("settings.appearance.color_theme")} }

            div { style: "display: grid; grid-template-columns: repeat(5, 1fr); gap: 10px; margin-bottom: 28px;",
                for (id, name, colors) in themes.iter().copied() {
                    {
                        let parts: Vec<&str> = colors.split(',').collect();
                        let (bg, primary, text) = (parts[0], parts[1], parts[2]);
                        let active = current_theme == id;
                        let border_style = if active {
                            format!("2px solid {primary}")
                        } else {
                            "2px solid transparent".to_string()
                        };
                        let id_owned = id.to_string();
                        let mut set_theme = set_theme.clone();
                        rsx! {
                            button {
                                key: "{id}",
                                title: "{name}",
                                style: "padding: 0; border-radius: var(--fsn-radius-md); \
                                        border: none; outline: {border_style}; \
                                        outline-offset: 2px; cursor: pointer; \
                                        overflow: hidden; display: flex; flex-direction: column;",
                                onclick: move |_| set_theme(id_owned.clone()),
                                // Color preview
                                div {
                                    style: "height: 44px; background: {bg}; \
                                            display: flex; align-items: center; \
                                            justify-content: center; gap: 5px; padding: 0 8px;",
                                    span {
                                        style: "width: 12px; height: 12px; border-radius: 50%; \
                                                background: {primary}; flex-shrink: 0;"
                                    }
                                    span {
                                        style: "width: 12px; height: 12px; border-radius: 50%; \
                                                background: {text}; opacity: 0.7; flex-shrink: 0;"
                                    }
                                }
                                // Label
                                div {
                                    style: "padding: 5px 4px; font-size: 10px; text-align: center; \
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

            // ── Section: Store Themes ──────────────────────────────────────
            if !store_themes.read().is_empty() {
                h3 { style: "margin-bottom: 12px; font-size: 16px;", {fsn_i18n::t("settings.appearance.installed_themes")} }

                // Remove confirm dialog
                if let Some(ref theme_id) = theme_remove_confirm.read().clone() {
                    div {
                        style: "position: fixed; inset: 0; background: rgba(0,0,0,0.5); \
                                display: flex; align-items: center; justify-content: center; z-index: 1000;",
                        div {
                            style: "background: var(--fsn-color-bg-surface); \
                                    border: 1px solid var(--fsn-color-border-default); \
                                    border-radius: var(--fsn-radius-lg); padding: 24px; \
                                    max-width: 380px; width: 100%;",
                            h3 { style: "margin: 0 0 12px 0;", {fsn_i18n::t("settings.appearance.remove_theme_title")} }
                            p {
                                style: "color: var(--fsn-color-text-muted); font-size: 14px; margin-bottom: 20px;",
                                {fsn_i18n::t("settings.appearance.remove_theme_body")}
                            }
                            div { style: "display: flex; gap: 8px; justify-content: flex-end;",
                                button {
                                    style: "padding: 8px 16px; background: var(--fsn-color-bg-surface); \
                                            border: 1px solid var(--fsn-color-border-default); \
                                            border-radius: var(--fsn-radius-md); cursor: pointer;",
                                    onclick: move |_| *theme_remove_confirm.write() = None,
                                    {fsn_i18n::t("actions.cancel")}
                                }
                                button {
                                    style: "padding: 8px 16px; background: var(--fsn-color-error, #ef4444); \
                                            color: white; border: none; \
                                            border-radius: var(--fsn-radius-md); cursor: pointer;",
                                    onclick: {
                                        let id = theme_id.clone();
                                        move |_| {
                                            let _ = PackageRegistry::remove(&id);
                                            store_themes.set(PackageRegistry::by_kind("theme"));
                                            *theme_remove_confirm.write() = None;
                                        }
                                    },
                                    {fsn_i18n::t("actions.remove")}
                                }
                            }
                        }
                    }
                }

                div { style: "margin-bottom: 28px;",
                    for pkg in store_themes.read().iter().cloned().collect::<Vec<_>>() {
                        {
                            let pkg_id   = pkg.id.clone();
                            let pkg_name = pkg.name.clone();
                            // Store themes ship a theme.css — read it on apply.
                            let css_path = pkg.file_path.clone();
                            let mut set_theme = set_theme.clone();
                            rsx! {
                                div {
                                    key: "{pkg_id}",
                                    style: "display: flex; align-items: center; gap: 12px; \
                                            padding: 10px 14px; margin-bottom: 6px; \
                                            background: var(--fsn-color-bg-surface); \
                                            border: 1px solid var(--fsn-color-border-default); \
                                            border-radius: var(--fsn-radius-md);",
                                    div {
                                        style: "flex: 1; font-size: 14px; font-weight: 500;",
                                        "{pkg_name}"
                                    }
                                    if css_path.is_some() {
                                        button {
                                            style: "padding: 6px 14px; background: var(--fsn-color-primary); \
                                                    color: white; border: none; \
                                                    border-radius: var(--fsn-radius-md); cursor: pointer; \
                                                    font-size: 12px;",
                                            onclick: {
                                                let path = css_path.clone().unwrap_or_default();
                                                move |_| {
                                                    if let Ok(css) = std::fs::read_to_string(&path) {
                                                        set_theme(format!("__custom__{}", css));
                                                    }
                                                }
                                            },
                                            {fsn_i18n::t("actions.apply")}
                                        }
                                    }
                                    button {
                                        style: "padding: 6px 12px; background: transparent; \
                                                border: 1px solid var(--fsn-color-error, #ef4444); \
                                                color: var(--fsn-color-error, #ef4444); \
                                                border-radius: var(--fsn-radius-md); cursor: pointer; \
                                                font-size: 12px;",
                                        onclick: {
                                            let id = pkg_id.clone();
                                            move |_| *theme_remove_confirm.write() = Some(id.clone())
                                        },
                                        {fsn_i18n::t("actions.remove")}
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Section: Window Chrome ─────────────────────────────────────
            h3 { style: "margin-bottom: 12px; font-size: 16px;", {fsn_i18n::t("settings.appearance.window_chrome")} }

            div { style: "display: flex; flex-direction: column; gap: 16px; margin-bottom: 28px;",

                // Animation toggle
                div { style: "display: flex; align-items: center; justify-content: space-between;",
                    div {
                        div { style: "font-size: 14px; font-weight: 500;", {fsn_i18n::t("settings.appearance.animations")} }
                        div { style: "font-size: 12px; color: var(--fsn-text-muted); margin-top: 2px;",
                            {fsn_i18n::t("settings.appearance.animations_hint")}
                        }
                    }
                    label {
                        style: "display: inline-flex; align-items: center; cursor: pointer; gap: 8px;",
                        input {
                            r#type: "checkbox",
                            checked: anim_enabled,
                            onchange: move |e| set_anim(e.checked()),
                            style: "width: 16px; height: 16px; cursor: pointer; accent-color: var(--fsn-primary);",
                        }
                        span { style: "font-size: 13px;", "{opacity_label}" }
                    }
                }

                // Window glass opacity
                div {
                    div { style: "display: flex; justify-content: space-between; align-items: baseline; margin-bottom: 6px;",
                        div { style: "font-size: 14px; font-weight: 500;", {fsn_i18n::t("settings.appearance.transparency")} }
                        span { style: "font-size: 12px; color: var(--fsn-text-muted);",
                            "{opacity_pct}%"
                        }
                    }
                    input {
                        r#type: "range",
                        min: "40",
                        max: "100",
                        value: "{opacity_pct}",
                        style: "width: 100%; accent-color: var(--fsn-primary);",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                set_opacity(v / 100.0);
                            }
                        },
                    }
                    div { style: "display: flex; justify-content: space-between; font-size: 11px; \
                                  color: var(--fsn-text-muted); margin-top: 4px;",
                        span { {fsn_i18n::t("settings.appearance.transparent")} }
                        span { {fsn_i18n::t("settings.appearance.opaque")} }
                    }
                }
            }

            // ── Section: Component Style ───────────────────────────────────
            h3 { style: "margin-top: 0; margin-bottom: 16px; font-size: 16px;", {fsn_i18n::t("settings.appearance.component_style")} }

            div { style: "display: flex; flex-direction: column; gap: 20px; margin-bottom: 28px;",
                {
                    let chrome_options: &[(&str, &str)] = &[("kde","KDE"),("macos","macOS"),("windows","Windows"),("minimal","Minimal")];
                    let btn_options:    &[(&str, &str)] = &[("rounded","Rounded"),("square","Square"),("pill","Pill"),("flat","Flat")];
                    let sidebar_options: &[(&str, &str)] = &[("solid","Solid"),("glass","Glass"),("transparent","Transparent")];

                    rsx! {
                        // Window Chrome
                        div {
                            div { style: "font-size: 14px; font-weight: 500; margin-bottom: 8px;", {fsn_i18n::t("settings.appearance.window_chrome")} }
                            div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                                for (id, label) in chrome_options.iter().copied() {
                                    {
                                        let active = chrome_style == id;
                                        let id_owned = id.to_string();
                                        let btn_style_s = if active {
                                            "padding: 6px 14px; border-radius: var(--fsn-radius-md); font-size: 13px; cursor: pointer; background: var(--fsn-primary); color: white; border: 1px solid var(--fsn-primary);"
                                        } else {
                                            "padding: 6px 14px; border-radius: var(--fsn-radius-md); font-size: 13px; cursor: pointer; background: var(--fsn-bg-surface); color: var(--fsn-text-primary); border: 1px solid var(--fsn-border);"
                                        };
                                        rsx! {
                                            button {
                                                key: "{id}",
                                                style: "{btn_style_s}",
                                                onclick: move |_| set_chrome_style(id_owned.clone()),
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Button Style
                        div {
                            div { style: "font-size: 14px; font-weight: 500; margin-bottom: 8px;", {fsn_i18n::t("settings.appearance.button_style")} }
                            div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                                for (id, label) in btn_options.iter().copied() {
                                    {
                                        let active = btn_style == id;
                                        let id_owned = id.to_string();
                                        let btn_style_s = if active {
                                            "padding: 6px 14px; border-radius: var(--fsn-radius-md); font-size: 13px; cursor: pointer; background: var(--fsn-primary); color: white; border: 1px solid var(--fsn-primary);"
                                        } else {
                                            "padding: 6px 14px; border-radius: var(--fsn-radius-md); font-size: 13px; cursor: pointer; background: var(--fsn-bg-surface); color: var(--fsn-text-primary); border: 1px solid var(--fsn-border);"
                                        };
                                        rsx! {
                                            button {
                                                key: "{id}",
                                                style: "{btn_style_s}",
                                                onclick: move |_| set_btn_style(id_owned.clone()),
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Sidebar Style
                        div {
                            div { style: "font-size: 14px; font-weight: 500; margin-bottom: 8px;", {fsn_i18n::t("settings.appearance.sidebar_style")} }
                            div { style: "display: flex; gap: 8px; flex-wrap: wrap;",
                                for (id, label) in sidebar_options.iter().copied() {
                                    {
                                        let active = sidebar_style == id;
                                        let id_owned = id.to_string();
                                        let btn_style_s = if active {
                                            "padding: 6px 14px; border-radius: var(--fsn-radius-md); font-size: 13px; cursor: pointer; background: var(--fsn-primary); color: white; border: 1px solid var(--fsn-primary);"
                                        } else {
                                            "padding: 6px 14px; border-radius: var(--fsn-radius-md); font-size: 13px; cursor: pointer; background: var(--fsn-bg-surface); color: var(--fsn-text-primary); border: 1px solid var(--fsn-border);"
                                        };
                                        rsx! {
                                            button {
                                                key: "{id}",
                                                style: "{btn_style_s}",
                                                onclick: move |_| set_sidebar_style(id_owned.clone()),
                                                "{label}"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // ── Section: Wallpaper ─────────────────────────────────────────
            h3 { style: "margin-bottom: 12px; font-size: 16px;", {fsn_i18n::t("settings.appearance.wallpaper")} }

            div { style: "display: flex; flex-direction: column; gap: 8px; margin-bottom: 28px;",
                // Solid color
                div { style: "display: flex; gap: 8px; align-items: center;",
                    span { style: "font-size: 13px; color: var(--fsn-color-text-muted); \
                                   min-width: 48px; flex-shrink: 0;",
                        {fsn_i18n::t("settings.appearance.wallpaper_color")}
                    }
                    input {
                        r#type: "color",
                        style: "width: 40px; height: 32px; border: 1px solid var(--fsn-border); \
                                border-radius: 4px; cursor: pointer; padding: 2px;",
                        oninput: move |e| {
                            let hex = e.value();
                            let css = format!("background-color: {};", hex);
                            if let Some(mut ctx) = wallpaper_ctx {
                                ctx.set(css);
                            }
                        },
                    }
                }

                // Gradient presets
                div { style: "display: flex; gap: 6px; flex-wrap: wrap; align-items: center;",
                    span { style: "font-size: 13px; color: var(--fsn-color-text-muted); \
                                   min-width: 48px; flex-shrink: 0;",
                        {fsn_i18n::t("settings.appearance.wallpaper_gradients")}
                    }
                    {
                        let presets: &[(&str, &str, &str)] = &[
                            ("Dark Blue", "#0f172a,#1e293b",  "linear-gradient(135deg, #0f172a, #1e293b)"),
                            ("Ocean",     "#0c4a6e,#0ea5e9",  "linear-gradient(135deg, #0c4a6e, #0369a1, #0ea5e9)"),
                            ("Forest",    "#052e16,#166534",  "linear-gradient(135deg, #052e16, #14532d, #166534)"),
                            ("Sunset",    "#450a0a,#c2410c",  "linear-gradient(135deg, #450a0a, #7c2d12, #c2410c)"),
                            ("Dusk",      "#1e1b4b,#4338ca",  "linear-gradient(135deg, #1e1b4b, #312e81, #4338ca)"),
                        ];
                        rsx! {
                            for (name, preview_colors, gradient_css) in presets.iter().copied() {
                                {
                                    let parts: Vec<&str> = preview_colors.split(',').collect();
                                    let (c1, c2) = (parts[0], parts[1]);
                                    let gradient_owned = gradient_css.to_string();
                                    rsx! {
                                        button {
                                            key: "{name}",
                                            title: "{name}",
                                            style: "width: 36px; height: 28px; border-radius: 4px; \
                                                    border: 1px solid var(--fsn-border); \
                                                    cursor: pointer; padding: 0; \
                                                    background: linear-gradient(135deg, {c1}, {c2}); \
                                                    flex-shrink: 0;",
                                            onclick: move |_| {
                                                let css = format!("background: {};", gradient_owned);
                                                if let Some(mut ctx) = wallpaper_ctx {
                                                    ctx.set(css);
                                                }
                                            },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // URL input
                div { style: "display: flex; gap: 8px;",
                    input {
                        r#type: "url",
                        placeholder: "https://example.com/wallpaper.jpg",
                        style: "flex: 1; padding: 8px 12px; \
                                border: 1px solid var(--fsn-border); \
                                border-radius: var(--fsn-radius-md); \
                                background: var(--fsn-bg-input); \
                                color: var(--fsn-text-primary); font-size: 13px;",
                        oninput: move |e| *wallpaper_url.write() = e.value(),
                    }
                    button {
                        style: "padding: 8px 14px; background: var(--fsn-primary); \
                                color: white; border: none; \
                                border-radius: var(--fsn-radius-md); cursor: pointer; \
                                font-size: 13px;",
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
                        "Load"
                    }
                }

                // File upload
                label {
                    style: "display: flex; align-items: center; gap: 8px; \
                            padding: 8px 12px; border: 1px dashed var(--fsn-border); \
                            border-radius: var(--fsn-radius-md); cursor: pointer; \
                            font-size: 13px; color: var(--fsn-text-secondary);",
                    input {
                        r#type: "file",
                        accept: "image/*",
                        style: "display: none;",
                        onchange: move |e| {
                            if let Some(engine) = e.files() {
                                if let Some(path) = engine.files().into_iter().next() {
                                    let css = format!(
                                        "background-image: url('file://{}'); \
                                         background-size: cover; background-position: center;",
                                        path
                                    );
                                    if let Some(mut ctx) = wallpaper_ctx {
                                        ctx.set(css);
                                    }
                                }
                            }
                        },
                    }
                    {fsn_i18n::t("settings.appearance.btn_upload_file")}
                }

                // Reset
                button {
                    style: "align-self: flex-start; padding: 6px 12px; font-size: 12px; \
                            background: transparent; \
                            border: 1px solid var(--fsn-border); \
                            border-radius: var(--fsn-radius-md); cursor: pointer; \
                            color: var(--fsn-text-muted);",
                    onclick: move |_| {
                        if let Some(mut ctx) = wallpaper_ctx {
                            ctx.set("background: linear-gradient(135deg, #0f172a, #1e293b);".to_string());
                        }
                        wallpaper_url.set(String::new());
                    },
                    {fsn_i18n::t("settings.appearance.btn_reset_wallpaper")}
                }
            }

            // ── Section: Theme Editor (B4) ─────────────────────────────────
            h3 { style: "margin-bottom: 8px; font-size: 16px;", {fsn_i18n::t("settings.appearance.custom_editor")} }
            p { style: "font-size: 12px; color: var(--fsn-text-muted); margin-bottom: 12px; line-height: 1.5;",
                "Paste CSS with "
                code { style: "font-family: var(--fsn-font-mono); \
                               background: var(--fsn-bg-elevated); \
                               padding: 1px 4px; border-radius: 3px;",
                    "--bg-base, --text-primary, --primary"
                }
                " etc. The "
                code { style: "font-family: var(--fsn-font-mono); \
                               background: var(--fsn-bg-elevated); \
                               padding: 1px 4px; border-radius: 3px;",
                    "--fsn-"
                }
                " "
                {fsn_i18n::t("settings.appearance.editor_prefix_hint")}
            }

            // Required vars hint
            div { style: "font-size: 11px; color: var(--fsn-text-muted); margin-bottom: 8px;",
                "Required: --bg-base, --bg-surface, --bg-elevated, --bg-card, --bg-input, --text-primary, --text-secondary, --text-muted, --primary, --primary-hover, --primary-text, --accent, --success, --warning, --error, --border, --border-focus"
            }

            // Editor textarea
            textarea {
                style: "width: 100%; height: 220px; padding: 12px; \
                        font-family: var(--fsn-font-mono); font-size: 12px; \
                        border: 1px solid var(--fsn-border); \
                        border-radius: var(--fsn-radius-md); \
                        background: var(--fsn-bg-input); \
                        color: var(--fsn-text-primary); \
                        resize: vertical; box-sizing: border-box;",
                placeholder: ":root {{\n  /* Required: backgrounds */\n  --bg-base: #0c1222;\n  --bg-surface: #162032;\n  --bg-elevated: #1e2d45;\n  --bg-card: #1a2538;\n  --bg-input: #0f1a2e;\n  /* Required: text */\n  --text-primary: #e8edf5;\n  --text-secondary: #a0b0c8;\n  --text-muted: #5a6e88;\n  /* Required: colors */\n  --primary: #4d8bf5;\n  --primary-hover: #3a78e8;\n  --primary-text: #ffffff;\n  --accent: #22d3ee;\n  /* Required: status */\n  --success: #34d399;\n  --warning: #fbbf24;\n  --error: #f87171;\n  /* Required: borders */\n  --border: rgba(148,170,200,0.18);\n  --border-focus: #4d8bf5;\n  /* Optional: effects */\n  --shadow: 0 4px 16px rgba(0,0,0,0.4);\n  --radius: 10px;\n  --transition: 0.18s ease;\n  --font-family: 'Inter', sans-serif;\n}}",
                oninput: move |e| {
                    custom_css.set(e.value());
                    editor_error.set(None);
                    editor_saved.set(false);
                },
            }

            // Editor error/success messages
            if let Some(err) = editor_error.read().as_ref() {
                div { style: "margin-top: 8px; padding: 8px 12px; \
                              background: var(--fsn-error-bg); \
                              border: 1px solid var(--fsn-error); \
                              border-radius: var(--fsn-radius-md); \
                              font-size: 12px; color: var(--fsn-error);",
                    "⚠ {err}"
                }
            }
            if *editor_saved.read() {
                div { style: "margin-top: 8px; padding: 8px 12px; \
                              background: var(--fsn-success-bg); \
                              border: 1px solid var(--fsn-success); \
                              border-radius: var(--fsn-radius-md); \
                              font-size: 12px; color: var(--fsn-success);",
                    {fsn_i18n::t("settings.appearance.editor_applied")}
                }
            }

            // Editor actions
            div { style: "display: flex; gap: 8px; margin-top: 10px;",
                button {
                    style: "padding: 8px 18px; background: var(--fsn-primary); color: white; \
                            border: none; border-radius: var(--fsn-radius-md); \
                            cursor: pointer; font-size: 13px; font-weight: 600;",
                    onclick: move |_| {
                        let css = custom_css.read().clone();
                        if css.trim().is_empty() {
                            editor_error.set(Some(fsn_i18n::t("settings.appearance.editor_empty_error")));
                            return;
                        }
                        let missing = validate_theme_vars(&css);
                        if !missing.is_empty() {
                            editor_error.set(Some(fsn_i18n::t_with(
                                "settings.appearance.editor_missing_vars",
                                &[("vars", &missing.join(", "))],
                            )));
                            return;
                        }
                        // Inject --fsn- prefix and apply as live preview.
                        let prefixed = prefix_theme_css(&css, "fsn");
                        if let Some(mut ctx) = theme_ctx {
                            ctx.set(format!("__custom__{}", prefixed));
                        }
                        editor_saved.set(true);
                        editor_error.set(None);
                    },
                    {fsn_i18n::t("settings.appearance.btn_apply_preview")}
                }
                button {
                    style: "padding: 8px 14px; background: transparent; \
                            border: 1px solid var(--fsn-border); \
                            border-radius: var(--fsn-radius-md); \
                            cursor: pointer; font-size: 13px; \
                            color: var(--fsn-text-secondary);",
                    onclick: move |_| {
                        custom_css.set(String::new());
                        editor_error.set(None);
                        editor_saved.set(false);
                    },
                    {fsn_i18n::t("actions.clear")}
                }
            }

            // Bottom spacing
            div { style: "height: 32px;" }
        }
    }
}

