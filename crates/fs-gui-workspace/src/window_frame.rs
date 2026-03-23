//! WindowFrame — FsObject implementation for windows.
//!
//! Features (per spec technik/ui-objekte.md):
//! - Drag at titlebar with fullscreen overlay (no event loss)
//! - Resize from all 8 handles (5px tolerance via CSS handles)
//! - Minimize → icon on desktop (handled in desktop.rs)
//! - Close: if has_unsaved_changes → UnsavedChangesDialog
//! - Window sidebar: universal Sidebar (left, overlay, hover-expand)
//! - Scrollable content area (.fs-scrollable)
//! - Double-click on titlebar → maximize / restore previous size+position
//! - Right-side help panel: universal HelpSidebarPanel (same as Desktop shell)
// WindowRenderFn is a fn pointer; #[component] derives PartialEq on props structs
// that include it — comparison is benign here since the same fn ptr is always passed.
#![allow(unpredictable_function_pointer_comparisons)]

use dioxus::prelude::*;
use fs_components::{Sidebar, SidebarItem, SidebarSide};

use crate::help_view::HelpSidebarPanel;
use crate::window::{OpenWindow, WindowButton, WindowId, WindowRenderFn, WindowSize};

// ── CSS constants ─────────────────────────────────────────────────────────────

/// Extra CSS injected for FsObject-specific styles (pulse animation, resize cursors).
pub const FSNOBJ_CSS: &str = r#"
/* ── Pulsing green dot for minimized window icons ─── */
@keyframes fs-pulse-green {
    0%   { opacity: 0; }
    40%  { opacity: 1; }
    60%  { opacity: 1; }
    100% { opacity: 0; }
}
.fs-pulse-dot {
    animation: fs-pulse-green 2.5s ease-in-out infinite;
    width: 8px; height: 8px;
    background: #22c55e;
    border-radius: 50%;
    position: absolute;
    top: -3px; right: -3px;
}


/* ── Resize handle cursors ──────────────────────────── */
.fs-resize-n,  .fs-resize-s  { cursor: ns-resize; }
.fs-resize-e,  .fs-resize-w  { cursor: ew-resize; }
.fs-resize-nw, .fs-resize-se { cursor: nwse-resize; }
.fs-resize-ne, .fs-resize-sw { cursor: nesw-resize; }

/* ── Minimized window icon on desktop ───────────────── */
.fs-win-icon {
    position: absolute;
    display: flex; flex-direction: column; align-items: center; gap: 4px;
    padding: 8px;
    cursor: pointer;
    pointer-events: all;
    user-select: none;
    width: 72px;
}
.fs-win-icon__box {
    position: relative;
    width: 48px; height: 48px;
    background: var(--fs-bg-elevated);
    border: 1px solid var(--fs-border);
    border-radius: 10px;
    display: flex; align-items: center; justify-content: center;
    font-size: 22px;
    box-shadow: var(--fs-shadow);
    transition: transform 120ms;
}
.fs-win-icon:hover .fs-win-icon__box { transform: scale(1.08); }
.fs-win-icon__label {
    font-size: 10px;
    color: var(--fs-text-secondary);
    text-align: center;
    max-width: 68px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    text-shadow: 0 1px 3px rgba(0,0,0,0.8);
}
"#;

// ── Props ─────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct WindowFrameProps {
    pub window:      OpenWindow,
    pub on_close:    EventHandler<WindowId>,
    pub on_focus:    EventHandler<WindowId>,
    pub on_minimize: EventHandler<WindowId>,
    pub on_maximize: EventHandler<WindowId>,
    /// Highest z_index among all open windows.
    /// The hover-boost only lifts this window above the current top,
    /// so intentionally focused windows are never displaced by hover.
    #[props(default = 0)]
    pub top_z:       u32,
}

// ── Resize direction ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
enum ResizeDir { N, S, E, W, NE, NW, SE, SW }

#[derive(Clone, Debug, PartialEq, Default)]
struct ResizeState {
    dir:      Option<ResizeDir>,
    start_mx: f64,
    start_my: f64,
    start_x:  f64,
    start_y:  f64,
    start_w:  f64,
    start_h:  f64,
}

// ── WindowFrame ───────────────────────────────────────────────────────────────

/// A draggable, resizable, closable window — the FsObject implementation.
#[component]
pub fn WindowFrame(props: WindowFrameProps) -> Element {
    let win = &props.window;
    let id  = win.id;

    // ── Position + size (in pixels) ───────────────────────────────────────────
    let init_pos = (100.0 + (id.0 % 8) as f64 * 40.0, 60.0 + (id.0 % 6) as f64 * 40.0);
    let _init_dim = win.size.initial_dimensions();

    let mut pos:  Signal<(f64, f64)> = use_signal(|| init_pos);
    let mut dim:  Signal<(f64, f64)> = use_signal(|| {
        match &win.size {
            WindowSize::Fixed { width, height }             => (*width, *height),
            WindowSize::Responsive { min_width, max_width } => ((min_width + max_width) / 2.0, 600.0),
            WindowSize::Fullscreen                          => (0.0, 0.0),
        }
    });

    // ── Drag state ────────────────────────────────────────────────────────────
    let mut dragging:   Signal<bool>           = use_signal(|| false);
    let mut drag_off:   Signal<(f64, f64)>     = use_signal(|| (0.0, 0.0));

    // ── Resize state ──────────────────────────────────────────────────────────
    let mut resize: Signal<ResizeState> = use_signal(ResizeState::default);

    // ── Unsaved-changes dialog state ──────────────────────────────────────────
    let mut close_requested: Signal<bool> = use_signal(|| false);

    // ── Hover-raise state ─────────────────────────────────────────────────────
    // While the mouse is over this window the z_index is temporarily boosted to
    // top_z + 1 — just enough to peek over the currently focused window.
    // This only has a visible effect when the window is NOT already the topmost.
    // A click (on_focus) makes the focus permanent via WindowManager.
    let mut hovered: Signal<bool> = use_signal(|| false);

    let (px, py)   = *pos.read();
    let (pw, ph)   = *dim.read();
    let is_dragging = *dragging.read();
    let is_resizing = resize.read().dir.is_some();
    let has_overlay = is_dragging || is_resizing;
    let is_max      = win.maximized;
    // Peek: temporarily above the current top, but only when not already on top.
    let effective_z = if *hovered.read() && win.z_index < props.top_z {
        props.top_z + 1
    } else {
        win.z_index
    };

    // ── Frame style ───────────────────────────────────────────────────────────
    let frame_style = if is_max {
        "position: absolute; left: 0; top: 0; width: 100%; height: 100%; \
         display: flex; flex-direction: column; \
         background: var(--fs-window-bg); \
         backdrop-filter: blur(16px) saturate(180%); -webkit-backdrop-filter: blur(16px) saturate(180%); \
         border: 1px solid var(--fs-window-border); \
         box-shadow: var(--fs-window-shadow); \
         pointer-events: all; \
         z-index: 9999; overflow: hidden;".to_string()
    } else {
        // Bug A fix: use `dim` signal (which tracks resize) instead of win.size spec.
        // Bug B fix: add max-height so window cannot overflow the viewport.
        let (w_style, h_style) = match &win.size {
            WindowSize::Fullscreen => ("100%".to_string(), "100%".to_string()),
            _                      => (format!("{pw}px"), format!("{ph}px")),
        };
        format!(
            "position: absolute; left: {px}px; top: {py}px; \
             width: {w_style}; height: {h_style}; \
             max-height: calc(100vh - 60px); \
             display: flex; flex-direction: column; \
             background: var(--fs-window-bg); \
             backdrop-filter: blur(16px) saturate(180%); -webkit-backdrop-filter: blur(16px) saturate(180%); \
             border: 1px solid var(--fs-window-border); \
             border-radius: 8px; \
             box-shadow: var(--fs-window-shadow); \
             pointer-events: all; \
             z-index: {}; overflow: visible;",
            effective_z
        )
    };

    // ── Overlay cursor ────────────────────────────────────────────────────────
    let overlay_cursor = if is_dragging {
        "grabbing"
    } else {
        match resize.read().dir {
            Some(ResizeDir::N)  | Some(ResizeDir::S)  => "ns-resize",
            Some(ResizeDir::E)  | Some(ResizeDir::W)  => "ew-resize",
            Some(ResizeDir::NW) | Some(ResizeDir::SE) => "nwse-resize",
            Some(ResizeDir::NE) | Some(ResizeDir::SW) => "nesw-resize",
            None => "default",
        }
    };

    // ── Has sidebar? ──────────────────────────────────────────────────────────
    let has_sidebar = !win.sidebar_items.is_empty();

    // Extract Copy values so closures don't need to move all of `props`
    let maximized          = win.maximized;
    let has_unsaved_changes = win.has_unsaved_changes;
    let render             = win.render;

    rsx! {
        // ── Window frame ───────────────────────────────────────────────────────
        div {
            class: "fs-window",
            style: "{frame_style}",
            // Click → permanent focus (z_index set to max by WindowManager)
            onmousedown: move |_| props.on_focus.call(id),
            // Hover → temporary z_index boost (local only, no WindowManager call)
            onmouseenter: move |_| hovered.set(true),
            onmouseleave: move |_| { if !has_overlay { hovered.set(false); } },

            // ── Resize handles (only when not maximized) ───────────────────
            if !is_max {
                ResizeHandles {
                    on_start: move |rs: ResizeState| resize.set(rs),
                    pos,
                    dim,
                }
            }

            // ── Titlebar ───────────────────────────────────────────────────
            div {
                class: "fs-window__titlebar",
                style: "display: flex; align-items: center; height: 40px; flex-shrink: 0; \
                        background: var(--fs-color-bg-sidebar, #0f172a); \
                        border-radius: 7px 7px 0 0; padding: 0 8px; \
                        cursor: grab; user-select: none; gap: 8px; position: relative;",

                onmousedown: move |evt: MouseEvent| {
                    evt.stop_propagation();
                    props.on_focus.call(id);
                    if !maximized {
                        let data = evt.data();
                        let c = data.client_coordinates();
                        let (px2, py2) = *pos.read();
                        drag_off.set((c.x - px2, c.y - py2));
                        dragging.set(true);
                    }
                },
                ondoubleclick: move |evt: MouseEvent| {
                    evt.stop_propagation();
                    props.on_maximize.call(id);
                },

                // App icon — task 3: explicit color so SVG currentColor is always visible
                if !win.icon.is_empty() {
                    span {
                        style: "font-size: 18px; flex-shrink: 0; display: flex; align-items: center; \
                                color: var(--fs-color-text-primary, #e2e8f0); z-index: 1;",
                        if win.icon.trim_start().starts_with("<svg") {
                            span { dangerous_inner_html: "{win.icon}" }
                        } else if win.icon.starts_with("http://") || win.icon.starts_with("https://") || win.icon.starts_with('/') {
                            img { src: "{win.icon}", width: "18", height: "18",
                                  style: "object-fit: contain; display: block;" }
                        } else {
                            "{win.icon}"
                        }
                    }
                }

                // Title — task 5: absolutely centered, larger font, human-readable name
                span {
                    style: "position: absolute; left: 0; right: 0; text-align: center; \
                            pointer-events: none; \
                            font-size: 15px; font-weight: 600; \
                            color: var(--fs-color-text-primary, #e2e8f0); \
                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap; \
                            padding: 0 90px;",
                    "{format_window_title(&win.title_key)}"
                }

                // Spacer pushes controls to the right
                div { style: "flex: 1;" }

                WindowControls {
                    closable: win.closable,
                    on_close: move |_| {
                        if has_unsaved_changes {
                            close_requested.set(true);
                        } else {
                            props.on_close.call(id);
                        }
                    },
                    on_minimize: move |_| props.on_minimize.call(id),
                    on_maximize: move |_| props.on_maximize.call(id),
                }
            }

            // ── Body: sidebar + content + help-panel ──────────────────────
            // position: relative so the absolute-positioned Sidebar panels anchor here.
            div {
                style: "display: flex; flex: 1; min-height: 0; overflow: hidden; position: relative;",

                // Left navigation sidebar — universal Sidebar, same component as the shell.
                // Only rendered when the window provides sidebar items.
                if has_sidebar {
                    Sidebar {
                        items: win.sidebar_items.iter()
                            .map(|i| SidebarItem::new(i.id.clone(), i.icon.clone(), i.label.clone()))
                            .collect::<Vec<_>>(),
                        active_id: win.active_sidebar_id.clone().unwrap_or_default(),
                        on_select: move |_item_id: String| {},
                        side: SidebarSide::Left,
                    }
                }

                // Content — padding clears the 44px tab-strips on each side.
                div {
                    class: if win.scrollable { "fs-window__content fs-scrollable" } else { "fs-window__content" },
                    style: {
                        let pl = if has_sidebar { "52px" } else { "20px" };
                        format!("flex: 1; padding: 20px 52px 20px {pl}; min-width: 0; \
                                 overflow-y: auto; overflow-x: hidden;")
                    },
                    WindowContent { render }
                }

                // Right-side help panel — same HelpSidebarPanel as the desktop shell.
                // Every window has it; content (Topics/Shortcuts/AI) is identical.
                HelpSidebarPanel {}
            }

            // ── Footer buttons ─────────────────────────────────────────────
            if !win.buttons.is_empty() {
                div {
                    class: "fs-window__footer",
                    style: "display: flex; justify-content: flex-end; gap: 8px; flex-shrink: 0; \
                            padding: 8px 16px; border-top: 1px solid var(--fs-color-border-default, #334155);",
                    for btn in &win.buttons {
                        WindowFooterButton {
                            button: btn.clone(),
                            on_close: {
                                let win_id = id;
                                move |should_close: bool| {
                                    if should_close { props.on_close.call(win_id); }
                                }
                            }
                        }
                    }
                }
            }

            // ── Unsaved Changes dialog (modal, inline) ─────────────────────
            if *close_requested.read() {
                UnsavedChangesDialog {
                    on_save: move |_| {
                        close_requested.set(false);
                        props.on_close.call(id);
                    },
                    on_discard: move |_| {
                        close_requested.set(false);
                        props.on_close.call(id);
                    },
                    on_cancel: move |_| {
                        close_requested.set(false);
                    },
                }
            }
        }

        // ── Fullscreen overlay (drag + resize) ─────────────────────────────
        if has_overlay {
            div {
                style: "position: fixed; inset: 0; z-index: 99999; pointer-events: all; cursor: {overlay_cursor};",
                onmousemove: move |evt: MouseEvent| {
                    let c = evt.data().client_coordinates();
                    if *dragging.read() {
                        let (ox, oy) = *drag_off.read();
                        // Clamp: titlebar (40px) must stay inside the viewport.
                        // py >= 0 keeps the titlebar reachable at the top edge.
                        let new_x = c.x - ox;
                        let new_y = (c.y - oy).max(0.0);
                        pos.set((new_x, new_y));
                    } else {
                        let rs = resize.read().clone();
                        if let Some(dir) = rs.dir {
                            let dx = c.x - rs.start_mx;
                            let dy = c.y - rs.start_my;
                            let min_w = 300.0_f64;
                            let min_h = 200.0_f64;
                            let (new_x, new_w, new_y, new_h) = match dir {
                                ResizeDir::E  => (rs.start_x, (rs.start_w + dx).max(min_w), rs.start_y, rs.start_h),
                                ResizeDir::W  => {
                                    let nw = (rs.start_w - dx).max(min_w);
                                    (rs.start_x + rs.start_w - nw, nw, rs.start_y, rs.start_h)
                                },
                                ResizeDir::S  => (rs.start_x, rs.start_w, rs.start_y, (rs.start_h + dy).max(min_h)),
                                ResizeDir::N  => {
                                    let nh = (rs.start_h - dy).max(min_h);
                                    (rs.start_x, rs.start_w, rs.start_y + rs.start_h - nh, nh)
                                },
                                ResizeDir::SE => (rs.start_x, (rs.start_w + dx).max(min_w), rs.start_y, (rs.start_h + dy).max(min_h)),
                                ResizeDir::SW => {
                                    let nw = (rs.start_w - dx).max(min_w);
                                    (rs.start_x + rs.start_w - nw, nw, rs.start_y, (rs.start_h + dy).max(min_h))
                                },
                                ResizeDir::NE => {
                                    let nh = (rs.start_h - dy).max(min_h);
                                    (rs.start_x, (rs.start_w + dx).max(min_w), rs.start_y + rs.start_h - nh, nh)
                                },
                                ResizeDir::NW => {
                                    let nw = (rs.start_w - dx).max(min_w);
                                    let nh = (rs.start_h - dy).max(min_h);
                                    (rs.start_x + rs.start_w - nw, nw, rs.start_y + rs.start_h - nh, nh)
                                },
                            };
                            pos.set((new_x, new_y));
                            dim.set((new_w, new_h));
                        }
                    }
                },
                onmouseup: move |_| {
                    dragging.set(false);
                    resize.set(ResizeState::default());
                },
            }
        }
    }
}

// ── Resize handles ────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct ResizeHandlesProps {
    on_start: EventHandler<ResizeState>,
    // Signals so closures always read the current position/size, not stale
    // captured values from the last render.
    pos: Signal<(f64, f64)>,
    dim: Signal<(f64, f64)>,
}

/// Eight invisible CSS handles (edges + corners) for resizing.
#[component]
fn ResizeHandles(props: ResizeHandlesProps) -> Element {
    macro_rules! handle {
        ($dir:expr, $style:literal) => {{
            let dir  = $dir;
            let pos_sig = props.pos;
            let dim_sig = props.dim;
            rsx! {
                div {
                    style: concat!("position: absolute; z-index: 200; ", $style),
                    onmousedown: move |evt: MouseEvent| {
                        evt.stop_propagation();
                        let c = evt.data().client_coordinates();
                        // Read current signal values inside the closure so we
                        // always get the latest position even after dragging.
                        let (px, py) = *pos_sig.read();
                        let (pw, ph) = *dim_sig.read();
                        props.on_start.call(ResizeState {
                            dir:      Some(dir),
                            start_mx: c.x,
                            start_my: c.y,
                            start_x:  px,
                            start_y:  py,
                            start_w:  pw,
                            start_h:  ph,
                        });
                    },
                }
            }
        }};
    }

    rsx! {
        // Edges
        {handle!(ResizeDir::N,  "top: -5px; left: 10px; right: 10px; height: 10px; cursor: ns-resize;")}
        {handle!(ResizeDir::S,  "bottom: -5px; left: 10px; right: 10px; height: 10px; cursor: ns-resize;")}
        {handle!(ResizeDir::W,  "left: -5px; top: 10px; bottom: 10px; width: 10px; cursor: ew-resize;")}
        {handle!(ResizeDir::E,  "right: -5px; top: 10px; bottom: 10px; width: 10px; cursor: ew-resize;")}
        // Corners (z-index higher so they win over edges)
        {handle!(ResizeDir::NW, "top: -5px; left: -5px; width: 14px; height: 14px; cursor: nwse-resize; z-index: 201;")}
        {handle!(ResizeDir::NE, "top: -5px; right: -5px; width: 14px; height: 14px; cursor: nesw-resize; z-index: 201;")}
        {handle!(ResizeDir::SW, "bottom: -5px; left: -5px; width: 14px; height: 14px; cursor: nesw-resize; z-index: 201;")}
        {handle!(ResizeDir::SE, "bottom: -5px; right: -5px; width: 14px; height: 14px; cursor: nwse-resize; z-index: 201;")}
    }
}


// ── Window controls (titlebar buttons) ───────────────────────────────────────

const ICON_MINIMIZE: &str = r#"<svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><line x1="2" y1="5" x2="8" y2="5"/></svg>"#;
const ICON_MAXIMIZE: &str = r#"<svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="6" height="6" rx="0.5"/></svg>"#;
const ICON_CLOSE:    &str = r#"<svg width="10" height="10" viewBox="0 0 10 10" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><line x1="2" y1="2" x2="8" y2="8"/><line x1="8" y1="2" x2="2" y2="8"/></svg>"#;

#[component]
fn WindowControls(
    closable:   bool,
    on_close:   EventHandler<MouseEvent>,
    on_minimize: EventHandler<MouseEvent>,
    on_maximize: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 2px;",
            button {
                class: "fs-window-btn",
                title: "Minimize",
                dangerous_inner_html: ICON_MINIMIZE,
                onmousedown: move |evt: MouseEvent| evt.stop_propagation(),
                onclick:     move |evt| { evt.stop_propagation(); on_minimize.call(evt); },
            }
            button {
                class: "fs-window-btn",
                title: "Maximize",
                dangerous_inner_html: ICON_MAXIMIZE,
                onmousedown: move |evt: MouseEvent| evt.stop_propagation(),
                onclick:     move |evt| { evt.stop_propagation(); on_maximize.call(evt); },
            }
            if closable {
                button {
                    class: "fs-window-btn fs-window-btn--close",
                    title: "Close",
                    dangerous_inner_html: ICON_CLOSE,
                    onmousedown: move |evt: MouseEvent| evt.stop_propagation(),
                    onclick:     move |evt| { evt.stop_propagation(); on_close.call(evt); },
                }
            }
        }
    }
}

// ── Unsaved Changes dialog ────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
struct UnsavedChangesDialogProps {
    on_save:    EventHandler<()>,
    on_discard: EventHandler<()>,
    on_cancel:  EventHandler<()>,
}

/// Modal dialog shown when closing a window with unsaved changes.
#[component]
fn UnsavedChangesDialog(props: UnsavedChangesDialogProps) -> Element {
    rsx! {
        // Backdrop
        div {
            style: "position: absolute; inset: 0; z-index: 9000; \
                    background: rgba(0,0,0,0.6); \
                    display: flex; align-items: center; justify-content: center;",
            onmousedown: move |evt: MouseEvent| evt.stop_propagation(),

            // Dialog box
            div {
                style: "background: var(--fs-bg-surface); \
                        border: 1px solid var(--fs-border); \
                        border-radius: 10px; \
                        padding: 24px 28px; \
                        min-width: 320px; max-width: 400px; \
                        box-shadow: var(--fs-shadow); \
                        display: flex; flex-direction: column; gap: 20px;",

                div {
                    style: "font-size: 15px; font-weight: 600; color: var(--fs-text-primary);",
                    "Unsaved Changes"
                }
                div {
                    style: "font-size: 13px; color: var(--fs-text-secondary);",
                    "You have unsaved changes. What would you like to do?"
                }
                div {
                    style: "display: flex; gap: 8px; justify-content: flex-end;",
                    button {
                        style: "padding: 6px 16px; border-radius: 6px; border: none; cursor: pointer; \
                                background: var(--fs-primary); color: #fff; font-size: 13px; font-family: inherit;",
                        onclick: move |_| props.on_save.call(()),
                        "Save"
                    }
                    button {
                        style: "padding: 6px 16px; border-radius: 6px; cursor: pointer; font-size: 13px; font-family: inherit; \
                                background: transparent; border: 1px solid var(--fs-border); color: var(--fs-text-primary);",
                        onclick: move |_| props.on_discard.call(()),
                        "Don't Save"
                    }
                    button {
                        style: "padding: 6px 16px; border-radius: 6px; cursor: pointer; font-size: 13px; font-family: inherit; \
                                background: transparent; border: 1px solid var(--fs-border); color: var(--fs-text-muted);",
                        onclick: move |_| props.on_cancel.call(()),
                        "Cancel"
                    }
                }
            }
        }
    }
}

// ── Footer button ─────────────────────────────────────────────────────────────

#[component]
fn WindowFooterButton(button: WindowButton, on_close: EventHandler<bool>) -> Element {
    match button {
        WindowButton::Ok => rsx! {
            button {
                class: "fs-btn fs-btn--primary",
                style: "padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; \
                        background: var(--fs-color-primary, #4d8bf5); color: #fff; font-size: 13px; font-family: inherit;",
                onclick: move |_| on_close.call(true),
                "OK"
            }
        },
        WindowButton::Cancel => rsx! {
            button {
                class: "fs-btn fs-btn--ghost",
                style: "padding: 6px 16px; border-radius: 4px; cursor: pointer; font-size: 13px; font-family: inherit; \
                        border: 1px solid var(--fs-color-border-default, #334155); \
                        background: transparent; color: var(--fs-color-text-primary, #e2e8f0);",
                onclick: move |_| on_close.call(true),
                "Cancel"
            }
        },
        WindowButton::Apply => rsx! {
            button {
                class: "fs-btn fs-btn--secondary",
                style: "padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: 13px; font-family: inherit; \
                        background: var(--fs-color-bg-active, #1e3a5f); color: var(--fs-color-text-primary, #e2e8f0);",
                onclick: move |_| on_close.call(false),
                "Apply"
            }
        },
        WindowButton::Custom { label_key, action_id: _ } => rsx! {
            button {
                class: "fs-btn fs-btn--ghost",
                style: "padding: 6px 16px; border-radius: 4px; cursor: pointer; font-size: 13px; font-family: inherit; \
                        border: 1px solid var(--fs-color-border-default, #334155); \
                        background: transparent; color: var(--fs-color-text-primary, #e2e8f0);",
                onclick: move |_| on_close.call(false),
                "{label_key}"
            }
        },
    }
}

// ── MinimizedWindowIcon ───────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct MinimizedWindowIconProps {
    pub window:     OpenWindow,
    pub pos_x:      f64,
    pub pos_y:      f64,
    pub on_restore: EventHandler<WindowId>,
    pub on_move:    EventHandler<(f64, f64)>,
}

/// Renders a minimized window as a draggable icon with pulsing green dot.
///
/// Bug C fix: restore-on-click now works even though the overlay intercepts mouseup.
/// We track drag_start and measure movement — if < 5px, treat as a click (restore).
#[component]
pub fn MinimizedWindowIcon(props: MinimizedWindowIconProps) -> Element {
    let id  = props.window.id;
    let mut icon_pos:   Signal<(f64, f64)> = use_signal(|| (props.pos_x, props.pos_y));
    let mut dragging:   Signal<bool>       = use_signal(|| false);
    let mut drag_off:   Signal<(f64, f64)> = use_signal(|| (0.0, 0.0));
    let mut drag_start: Signal<(f64, f64)> = use_signal(|| (0.0, 0.0));

    let (ix, iy) = *icon_pos.read();
    let is_dragging = *dragging.read();
    let title = props.window.title_key.trim_start_matches("app-");
    let icon  = &props.window.icon;

    rsx! {
        div {
            class: "fs-win-icon",
            style: "left: {ix}px; top: {iy}px;",

            onmousedown: move |evt: MouseEvent| {
                let c = evt.data().client_coordinates();
                let (cx, cy) = *icon_pos.read();
                drag_off.set((c.x - cx, c.y - cy));
                drag_start.set((c.x, c.y));
                dragging.set(true);
            },

            div {
                class: "fs-win-icon__box",
                if icon.trim_start().starts_with("<svg") {
                    span { dangerous_inner_html: "{icon}" }
                } else if icon.starts_with("http://") || icon.starts_with("https://") || icon.starts_with('/') {
                    img { src: "{icon}", width: "24", height: "24",
                          style: "object-fit: contain; display: block;" }
                } else {
                    "{icon}"
                }
                span { class: "fs-pulse-dot" }
            }
            span {
                class: "fs-win-icon__label",
                "{title}"
            }
        }

        // Overlay is shown as soon as mousedown fires (dragging = true).
        // On mouseup: measure how far the mouse moved. < 5px = click → restore.
        if is_dragging {
            div {
                style: "position: fixed; inset: 0; z-index: 99999; pointer-events: all; cursor: grabbing;",
                onmousemove: move |evt: MouseEvent| {
                    let c = evt.data().client_coordinates();
                    let (ox, oy) = *drag_off.read();
                    icon_pos.set((c.x - ox, c.y - oy));
                },
                onmouseup: move |evt: MouseEvent| {
                    dragging.set(false);
                    let c = evt.data().client_coordinates();
                    let (sx, sy) = *drag_start.read();
                    let moved = ((c.x - sx).powi(2) + (c.y - sy).powi(2)).sqrt();
                    if moved < 5.0 {
                        props.on_restore.call(id);
                    } else {
                        props.on_move.call(*icon_pos.read());
                    }
                },
            }
        }
    }
}


// ── WindowContent ─────────────────────────────────────────────────────────────

/// Thin wrapper that calls a `WindowRenderFn` inside its own Dioxus scope.
///
/// Wrapping the render call in a component (rather than calling the fn directly)
/// gives the app a stable scope: `use_signal` and other hooks inside the app live
/// in `WindowContent`'s scope and survive re-renders of the surrounding `WindowFrame`.
/// Because `render` is a bare function pointer (`Copy + PartialEq`), Dioxus can
/// memoize this component and will not re-create the app scope on every frame.
#[component]
fn WindowContent(render: WindowRenderFn) -> Element {
    (render)()
}

// ── format_window_title ────────────────────────────────────────────────────────

/// Converts a raw `title_key` (e.g. "app-store", "app-language-manager") into a
/// human-readable window title shown in the titlebar.
fn format_window_title(title_key: &str) -> String {
    let id = title_key.strip_prefix("app-").unwrap_or(title_key);
    match id {
        "store"            => "Store".to_string(),
        "settings"         => "Settings".to_string(),
        "container"        => "Container Manager".to_string(),
        "bot-manager"      => "Bot Manager".to_string(),
        "language-manager" => "Language Manager".to_string(),
        "theme-manager"    => "Theme Manager".to_string(),
        "icons-manager"    => "Icons Manager".to_string(),
        "managers"         => "Managers".to_string(),
        "profile"          => "Profile".to_string(),
        "browser"          => "Browser".to_string(),
        "lenses"           => "Lenses".to_string(),
        "builder"          => "Builder".to_string(),
        "tasks"            => "Tasks".to_string(),
        "ai"               => "AI Assistant".to_string(),
        "help"             => "Help".to_string(),
        other              => {
            // Convert kebab-case to Title Case as fallback for unknown apps.
            other.split('-')
                .map(|word| {
                    let mut chars = word.chars();
                    match chars.next() {
                        None    => String::new(),
                        Some(c) => c.to_uppercase().to_string() + chars.as_str(),
                    }
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}
