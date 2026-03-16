/// WindowFrame — renders an open window with titlebar, controls, and drag support.
use dioxus::prelude::*;

use crate::window::{Window, WindowButton, WindowId, WindowSize};

#[derive(Props, Clone, PartialEq)]
pub struct WindowFrameProps {
    pub window: Window,
    pub on_close: EventHandler<WindowId>,
    pub on_focus: EventHandler<WindowId>,
    pub on_minimize: EventHandler<WindowId>,
    pub on_maximize: EventHandler<WindowId>,
    pub children: Element,
}

/// A draggable, resizable window frame rendered over the desktop.
#[component]
pub fn WindowFrame(props: WindowFrameProps) -> Element {
    let win = &props.window;
    let id = win.id;

    // Drag state: offset from top-left corner when drag started
    let mut drag_offset: Signal<Option<(f64, f64)>> = use_signal(|| None);
    let mut position: Signal<(f64, f64)> = use_signal(|| (100.0 + (id.0 % 8) as f64 * 30.0, 60.0 + (id.0 % 6) as f64 * 30.0));

    let (w_css, h_css) = match &win.size {
        WindowSize::Fixed { width, height } => (format!("{}px", width), format!("{}px", height)),
        WindowSize::Responsive { min_width, max_width } => (
            format!("clamp({}px, 60vw, {}px)", min_width, max_width),
            "auto".into(),
        ),
        WindowSize::Fullscreen => ("100%".into(), "100%".into()),
    };

    let (left, top) = *position.read();

    let frame_style = if win.maximized {
        "position: absolute; left: 0; top: 0; width: 100%; height: 100%; \
         display: flex; flex-direction: column; \
         background: var(--fsn-window-bg); \
         backdrop-filter: blur(16px) saturate(180%); -webkit-backdrop-filter: blur(16px) saturate(180%); \
         border: 1px solid var(--fsn-window-border); \
         box-shadow: var(--fsn-window-shadow); \
         z-index: 9999;".to_string()
    } else {
        format!(
            "position: absolute; left: {}px; top: {}px; width: {}; min-height: {}; \
             display: flex; flex-direction: column; \
             background: var(--fsn-window-bg); \
             backdrop-filter: blur(16px) saturate(180%); -webkit-backdrop-filter: blur(16px) saturate(180%); \
             border: 1px solid var(--fsn-window-border); \
             border-radius: 8px; \
             box-shadow: var(--fsn-window-shadow); \
             z-index: {};",
            left, top, w_css, h_css, win.z_index
        )
    };

    rsx! {
        div {
            class: "fsd-window",
            style: frame_style,
            onmousedown: move |_| {
                props.on_focus.call(id);
            },

            // ── Titlebar ─────────────────────────────────────────────────────
            div {
                class: "fsd-window__titlebar",
                style: "display: flex; align-items: center; height: 36px; \
                        background: var(--fsn-color-bg-sidebar, #0f172a); \
                        border-radius: 7px 7px 0 0; padding: 0 8px; \
                        cursor: grab; user-select: none; gap: 8px;",

                onmousedown: move |evt: MouseEvent| {
                    evt.stop_propagation();
                    props.on_focus.call(id);
                    if !props.window.maximized {
                        let data = evt.data();
                        let client = data.client_coordinates();
                        let (px, py) = *position.read();
                        drag_offset.set(Some((client.x - px, client.y - py)));
                    }
                },
                onmousemove: move |evt: MouseEvent| {
                    if let Some((ox, oy)) = *drag_offset.read() {
                        let data = evt.data();
                        let client = data.client_coordinates();
                        position.set((client.x - ox, client.y - oy));
                    }
                },
                onmouseup: move |_| {
                    drag_offset.set(None);
                },
                onmouseleave: move |_| {
                    drag_offset.set(None);
                },

                // Window title
                span {
                    style: "flex: 1; font-size: 13px; color: var(--fsn-color-text-primary, #e2e8f0); \
                            font-weight: 500; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{win.title_key}"
                }

                // Window controls
                WindowControls {
                    closable: win.closable,
                    on_close: move |_| props.on_close.call(id),
                    on_minimize: move |_| props.on_minimize.call(id),
                    on_maximize: move |_| props.on_maximize.call(id),
                }
            }

            // ── Content area ─────────────────────────────────────────────────
            div {
                class: "fsd-window__content",
                style: if win.scrollable {
                    "flex: 1; overflow-y: auto; padding: 16px;"
                } else {
                    "flex: 1; overflow: hidden; padding: 16px;"
                },
                {props.children}
            }

            // ── Footer buttons ────────────────────────────────────────────────
            if !win.buttons.is_empty() {
                div {
                    class: "fsd-window__footer",
                    style: "display: flex; justify-content: flex-end; gap: 8px; \
                            padding: 8px 16px; border-top: 1px solid var(--fsn-color-border-default, #334155);",
                    for btn in &win.buttons {
                        WindowFooterButton {
                            button: btn.clone(),
                            on_close: {
                                let win_id = id;
                                move |should_close: bool| {
                                    if should_close {
                                        props.on_close.call(win_id);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── KDE/Breeze window control SVG icons ──────────────────────────────────────

const ICON_MINIMIZE: &str = r#"<svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><line x1="2" y1="5" x2="8" y2="5"/></svg>"#;
const ICON_MAXIMIZE: &str = r#"<svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="6" height="6" rx="0.5"/></svg>"#;
const ICON_CLOSE: &str    = r#"<svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><line x1="2" y1="2" x2="8" y2="8"/><line x1="8" y1="2" x2="2" y2="8"/></svg>"#;

/// The three window control buttons — KDE/Breeze style: rectangular with SVG icons.
/// Order: Minimize · Maximize · Close (right-aligned, close turns red on hover).
#[component]
fn WindowControls(
    closable: bool,
    on_close: EventHandler<MouseEvent>,
    on_minimize: EventHandler<MouseEvent>,
    on_maximize: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 2px;",

            // Minimize
            button {
                class: "fsd-window-btn",
                title: "Minimize",
                dangerous_inner_html: ICON_MINIMIZE,
                onmousedown: move |evt: MouseEvent| evt.stop_propagation(),
                onclick: move |evt| {
                    evt.stop_propagation();
                    on_minimize.call(evt);
                },
            }

            // Maximize / restore
            button {
                class: "fsd-window-btn",
                title: "Maximize",
                dangerous_inner_html: ICON_MAXIMIZE,
                onmousedown: move |evt: MouseEvent| evt.stop_propagation(),
                onclick: move |evt| {
                    evt.stop_propagation();
                    on_maximize.call(evt);
                },
            }

            // Close
            if closable {
                button {
                    class: "fsd-window-btn fsd-window-btn--close",
                    title: "Close",
                    dangerous_inner_html: ICON_CLOSE,
                    onmousedown: move |evt: MouseEvent| evt.stop_propagation(),
                    onclick: move |evt| {
                        evt.stop_propagation();
                        on_close.call(evt);
                    },
                }
            }
        }
    }
}

/// A single footer action button.
#[component]
fn WindowFooterButton(button: WindowButton, on_close: EventHandler<bool>) -> Element {
    match button {
        WindowButton::Ok => rsx! {
            button {
                class: "fsd-btn fsd-btn--primary",
                style: "padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; \
                        background: var(--fsn-color-primary, #4d8bf5); color: #fff; font-size: 13px;",
                onclick: move |_| on_close.call(true),
                "OK"
            }
        },
        WindowButton::Cancel => rsx! {
            button {
                class: "fsd-btn fsd-btn--ghost",
                style: "padding: 6px 16px; border-radius: 4px; border: 1px solid var(--fsn-color-border-default, #334155); \
                        cursor: pointer; background: transparent; color: var(--fsn-color-text-primary, #e2e8f0); font-size: 13px;",
                onclick: move |_| on_close.call(true),
                "Cancel"
            }
        },
        WindowButton::Apply => rsx! {
            button {
                class: "fsd-btn fsd-btn--secondary",
                style: "padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; \
                        background: var(--fsn-color-bg-active, #1e3a5f); color: var(--fsn-color-text-primary, #e2e8f0); font-size: 13px;",
                onclick: move |_| on_close.call(false),
                "Apply"
            }
        },
        WindowButton::Custom { label_key, action_id: _ } => rsx! {
            button {
                class: "fsd-btn fsd-btn--ghost",
                style: "padding: 6px 16px; border-radius: 4px; border: 1px solid var(--fsn-color-border-default, #334155); \
                        cursor: pointer; background: transparent; color: var(--fsn-color-text-primary, #e2e8f0); font-size: 13px;",
                onclick: move |_| on_close.call(false),
                "{label_key}"
            }
        },
    }
}
