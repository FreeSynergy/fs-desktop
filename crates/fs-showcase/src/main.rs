//! fs-showcase — Component gallery for FreeSynergy.Desktop.
//!
//! Only meaningful in debug builds. In release mode it exits immediately.

fn main() {
    #[cfg(not(debug_assertions))]
    {
        eprintln!("fs-showcase is a debug-only tool. Run with `cargo run` (without --release).");
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

    tracing::info!("Starting fs-showcase");

    #[cfg(feature = "desktop")]
    fs_components::launch_desktop(
        fs_components::DesktopConfig::new()
            .with_title("FreeSynergy \u{2013} Component Showcase")
            .with_size(1400.0, 900.0),
        showcase_app,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// All showcase code is debug-only.
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(debug_assertions)]
mod showcase {
    use dioxus::prelude::*;
    use fs_components::*;

    // ── Root ──────────────────────────────────────────────────────────────────

    #[allow(non_snake_case)]
    pub fn showcase_app() -> Element {
        let mut btn_loading = use_signal(|| false);
        let mut input_val   = use_signal(|| String::new());
        let mut checked     = use_signal(|| false);
        let mut select_val  = use_signal(|| "md".to_string());

        rsx! {
            style {
                "*, *::before, *::after {{ box-sizing: border-box; }}
                 body {{ margin: 0; background: #0d1117; color: #e6edf3;
                         font-family: 'JetBrains Mono', 'Fira Code', monospace; font-size: 14px; }}
                 @keyframes fs-spin {{ from {{ transform: rotate(0deg); }} to {{ transform: rotate(360deg); }} }}"
            }

            ToastProvider {
                div { style: "display: flex; min-height: 100vh;",

                    nav {
                        style: "width: 200px; flex-shrink: 0; padding: 24px 16px;
                                background: #161b22; border-right: 1px solid #30363d;",
                        h2 { style: "margin: 0 0 24px; font-size: 11px; color: #00BCD4;
                                     text-transform: uppercase; letter-spacing: 0.08em;",
                            "FreeSynergy" }
                        p { style: "margin: 0; font-size: 10px; color: #8b949e;",
                            "Component Showcase" }
                    }

                    main { style: "flex: 1; padding: 32px; overflow: auto;",

                        h1 { style: "margin: 0 0 8px; color: #00BCD4; font-size: 20px;",
                            "FreeSynergy — Component Showcase" }
                        p { style: "margin: 0 0 40px; color: #8b949e; font-size: 13px;",
                            "All components from fs-components, rendered with desktop feature." }

                        // Buttons
                        ShowcaseSection { title: "Buttons",
                            div { style: "display: flex; flex-wrap: wrap; gap: 12px; align-items: center;",
                                Button { variant: ButtonVariant::Primary,   "Primary" }
                                Button { variant: ButtonVariant::Secondary, "Secondary" }
                                Button { variant: ButtonVariant::Ghost,     "Ghost" }
                                Button { variant: ButtonVariant::Danger,    "Danger" }
                                Button { variant: ButtonVariant::Primary, size: ButtonSize::Sm, "Small" }
                                Button { variant: ButtonVariant::Primary, size: ButtonSize::Lg, "Large" }
                                Button {
                                    variant: ButtonVariant::Primary,
                                    loading: *btn_loading.read(),
                                    onclick: move |_| { let v = *btn_loading.read(); *btn_loading.write() = !v; },
                                    "Toggle Loading"
                                }
                                Button { variant: ButtonVariant::Primary, disabled: true, "Disabled" }
                            }
                        }

                        // Badges
                        ShowcaseSection { title: "Badges",
                            div { style: "display: flex; flex-wrap: wrap; gap: 8px; align-items: center;",
                                Badge { "Default" }
                                Badge { variant: BadgeVariant::Success, "Success" }
                                Badge { variant: BadgeVariant::Warning, "Warning" }
                                Badge { variant: BadgeVariant::Error,   "Error" }
                                Badge { variant: BadgeVariant::Info,    "Info" }
                            }
                        }

                        // Cards
                        ShowcaseSection { title: "Cards",
                            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px;",
                                Card {
                                    p { style: "margin: 0; color: #e6edf3;", "Standard card" }
                                    p { style: "margin: 8px 0 0; font-size: 12px; color: #8b949e;",
                                        "Background: bg-surface with border." }
                                }
                                Card { glass: true,
                                    p { style: "margin: 0; color: #e6edf3;", "Glass card" }
                                    p { style: "margin: 8px 0 0; font-size: 12px; color: #8b949e;",
                                        "Glassmorphism with backdrop-filter." }
                                }
                            }
                        }

                        // Spinner + Divider
                        ShowcaseSection { title: "Spinner + Divider",
                            div { style: "display: flex; gap: 24px; align-items: center;",
                                Spinner { size: 20 }
                                Spinner { size: 32 }
                                Spinner { size: 48 }
                            }
                            Divider { margin: "16px 0".to_string() }
                            Divider { label: "OR".to_string(), margin: "16px 0".to_string() }
                        }

                        // Tooltip
                        ShowcaseSection { title: "Tooltip",
                            div { style: "display: flex; gap: 16px; padding: 24px 0;",
                                Tooltip { text: "Hover me!".to_string(),
                                    Button { variant: ButtonVariant::Ghost, "Hover for tooltip" }
                                }
                            }
                        }

                        // Form controls
                        ShowcaseSection { title: "Form Controls",
                            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 20px; max-width: 600px;",
                                FormField {
                                    label: "Text Input".to_string(), field_id: "showcase-input".to_string(),
                                    hint: Some("Type something…".to_string()),
                                    Input {
                                        id: "showcase-input".to_string(),
                                        value: input_val.read().clone(),
                                        placeholder: Some("Placeholder…".to_string()),
                                        oninput: move |e: FormEvent| { *input_val.write() = e.value(); },
                                    }
                                }
                                FormField {
                                    label: "Select".to_string(), field_id: "showcase-select".to_string(),
                                    Select {
                                        id: "showcase-select".to_string(),
                                        value: select_val.read().clone(),
                                        options: vec![
                                            SelectOption::new("sm", "Small"),
                                            SelectOption::new("md", "Medium"),
                                            SelectOption::new("lg", "Large"),
                                        ],
                                        onchange: move |e: FormEvent| { *select_val.write() = e.value(); },
                                    }
                                }
                                FormField {
                                    label: "Email (required)".to_string(), field_id: "showcase-email".to_string(),
                                    required: true, error: "Please enter a valid email.".to_string(),
                                    Input { id: "showcase-email".to_string(), r#type: "email".to_string(), value: "bad-input".to_string() }
                                }
                                FormField {
                                    label: "Textarea".to_string(), field_id: "showcase-textarea".to_string(),
                                    Textarea {
                                        id: "showcase-textarea".to_string(),
                                        placeholder: Some("Enter description…".to_string()),
                                        rows: 3,
                                    }
                                }
                                div {
                                    Checkbox {
                                        id: "showcase-check".to_string(),
                                        checked: *checked.read(),
                                        label: "I agree to the terms".to_string(),
                                        onchange: move |e: FormEvent| { *checked.write() = e.value() == "true"; },
                                    }
                                }
                            }
                        }

                        // Toast
                        ShowcaseSection { title: "Toast Notifications",
                            div { style: "display: flex; flex-wrap: wrap; gap: 8px;",
                                ToastTrigger { level: "info",    label: "Info Toast" }
                                ToastTrigger { level: "success", label: "Success Toast" }
                                ToastTrigger { level: "warning", label: "Warning Toast" }
                                ToastTrigger { level: "error",   label: "Error Toast" }
                            }
                        }
                    }
                }
            }
        }
    }

    // ── ShowcaseSection ───────────────────────────────────────────────────────

    #[component]
    fn ShowcaseSection(title: String, children: Element) -> Element {
        rsx! {
            section { style: "margin-bottom: 40px;",
                h2 { style: "margin: 0 0 16px; font-size: 13px; font-weight: 600; \
                              color: #8b949e; text-transform: uppercase; letter-spacing: 0.06em; \
                              border-bottom: 1px solid #30363d; padding-bottom: 8px;",
                    "{title}" }
                {children}
            }
        }
    }

    // ── ToastTrigger ──────────────────────────────────────────────────────────

    #[component]
    fn ToastTrigger(level: String, label: String) -> Element {
        let mut toast = use_toast();
        let lv  = level.clone();
        let lbl = label.clone();

        rsx! {
            Button {
                variant: match level.as_str() {
                    "success" => ButtonVariant::Primary,
                    "danger" | "error" => ButtonVariant::Danger,
                    _ => ButtonVariant::Ghost,
                },
                onclick: move |_| {
                    let msg = match lv.as_str() {
                        "info"    => ToastMessage::info(lbl.clone()),
                        "success" => ToastMessage::success(lbl.clone()),
                        "warning" => ToastMessage::warning(lbl.clone()),
                        _         => ToastMessage::error(lbl.clone(), "Something went wrong."),
                    };
                    toast.push(msg);
                },
                "{label}"
            }
        }
    }
}

#[cfg(debug_assertions)]
use showcase::showcase_app;
