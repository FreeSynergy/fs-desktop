/// `WebDesktop` — mobile-first shell layout for the web target.
///
/// Layout:
///   - `TopBar` (always visible):  [≡ Menu]  `FreeSynergy`  [Bell]  [User Admin]
///   - Content area: embedded app, fills remaining space, scrollable
///   - Taskbar (bottom): Fixed by default, slides up on tap/click, auto-hide optional
///
/// Taskbar states: Fixed → `SlideUp` → Hidden (cycles on tap of drag handle).
use dioxus::prelude::*;

use crate::icons::ICON_BELL;

use crate::taskbar::AppEntry;

// ── Taskbar state ─────────────────────────────────────────────────────────────

/// Visibility state for the bottom taskbar in the web layout.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum WebTaskbarState {
    /// Always visible at the bottom.
    #[default]
    Fixed,
    /// Slid up to reveal full content, taskbar still reachable via handle.
    SlideUp,
    /// Fully hidden — only the pull handle peeks.
    Hidden,
}

impl WebTaskbarState {
    /// Cycle: Fixed → `SlideUp` → Hidden → Fixed.
    #[must_use]
    pub fn cycle(self) -> Self {
        match self {
            Self::Fixed => Self::SlideUp,
            Self::SlideUp => Self::Hidden,
            Self::Hidden => Self::Fixed,
        }
    }

    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Fixed => "Fixed",
            Self::SlideUp => "Slide Up",
            Self::Hidden => "Hidden",
        }
    }
}

// ── Props ─────────────────────────────────────────────────────────────────────

#[derive(Props, Clone, PartialEq)]
pub struct WebDesktopProps {
    /// Apps shown in the bottom taskbar launcher.
    #[props(default)]
    pub apps: Vec<AppEntry>,

    /// Current user display name.
    #[props(default = "Admin".to_string())]
    pub user_name: String,

    /// Notification count badge (0 = hidden).
    #[props(default)]
    pub notification_count: u32,

    /// Called when the menu button (≡) is clicked.
    #[props(default)]
    pub on_menu: Option<EventHandler<()>>,

    /// Called when a taskbar app is launched.
    #[props(default)]
    pub on_launch: Option<EventHandler<String>>,

    /// Children rendered in the content area.
    children: Element,
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Root shell for the web target.
/// Replaces the CSS-grid desktop with a topbar + scrollable content + slide-up taskbar.
#[component]
pub fn WebDesktop(props: WebDesktopProps) -> Element {
    let mut taskbar_state: Signal<WebTaskbarState> = use_signal(WebTaskbarState::default);

    let taskbar_bottom = match *taskbar_state.read() {
        WebTaskbarState::Fixed => "0px",
        WebTaskbarState::SlideUp => "calc(48px - 100%)", // only handle visible
        WebTaskbarState::Hidden => "-48px",
    };

    rsx! {
        div {
            class: "fs-web-desktop",
            style: "display: flex; flex-direction: column; height: 100dvh; \
                    background: var(--fs-bg-base, #0a0f1a); overflow: hidden; \
                    position: relative;",

            // ── TopBar ─────────────────────────────────────────────────────
            WebTopBar {
                user_name: props.user_name.clone(),
                notification_count: props.notification_count,
                on_menu: props.on_menu,
            }

            // ── Content area ───────────────────────────────────────────────
            div {
                class: "fs-web-desktop__content fs-scrollable",
                style: "flex: 1; overflow-y: auto; overflow-x: hidden; \
                        padding-bottom: 56px; /* leave room for taskbar */",
                {props.children}
            }

            // ── Bottom taskbar ─────────────────────────────────────────────
            div {
                class: "fs-web-desktop__taskbar",
                style: "position: fixed; bottom: {taskbar_bottom}; left: 0; right: 0; \
                        z-index: 200; transition: bottom 280ms cubic-bezier(0.4,0,0.2,1);",

                // Drag / tap handle
                div {
                    style: "display: flex; justify-content: center; align-items: center; \
                            height: 20px; background: var(--fs-bg-sidebar); \
                            border-top: 1px solid var(--fs-border); cursor: pointer; \
                            border-radius: 8px 8px 0 0;",
                    onclick: move |_| {
                        let next = taskbar_state.read().cycle();
                        *taskbar_state.write() = next;
                    },
                    div {
                        style: "width: 36px; height: 4px; border-radius: 2px; \
                                background: var(--fs-text-muted, #64748b);",
                    }
                }

                // Taskbar body
                WebTaskbarBody {
                    apps: props.apps.clone(),
                    on_launch: props.on_launch,
                }
            }
        }
    }
}

// ── TopBar ────────────────────────────────────────────────────────────────────

#[component]
fn WebTopBar(
    user_name: String,
    notification_count: u32,
    on_menu: Option<EventHandler<()>>,
) -> Element {
    let initial: String = user_name
        .chars()
        .next()
        .map_or_else(|| "?".into(), |c| c.to_uppercase().to_string());

    rsx! {
        header {
            class: "fs-web-topbar",
            style: "display: flex; align-items: center; height: 48px; \
                    padding: 0 12px; gap: 8px; flex-shrink: 0; \
                    background: var(--fs-bg-sidebar, #0a0f1a); \
                    border-bottom: 1px solid var(--fs-border, rgba(148,170,200,0.18)); \
                    z-index: 100;",

            // Menu button
            button {
                style: "background: none; border: none; cursor: pointer; padding: 6px 8px; \
                        border-radius: var(--fs-radius-sm); font-size: 18px; line-height: 1; \
                        color: var(--fs-text-secondary);",
                onclick: move |_| {
                    if let Some(h) = &on_menu { h.call(()); }
                },
                "≡"
            }

            // Brand
            div {
                style: "flex: 1; display: flex; align-items: center; justify-content: center;",
                span {
                    style: "font-size: 15px; font-weight: 700; color: var(--fs-primary); \
                            letter-spacing: -0.01em;",
                    "FreeSynergy"
                }
            }

            // Notification bell
            div { style: "position: relative;",
                button {
                    style: "background: none; border: none; cursor: pointer; padding: 6px 8px; \
                            border-radius: var(--fs-radius-sm); display: flex; align-items: center; \
                            color: var(--fs-text-secondary);",
                    span { dangerous_inner_html: ICON_BELL }
                }
                if notification_count > 0 {
                    div {
                        style: "position: absolute; top: 2px; right: 2px; \
                                background: var(--fs-danger, #ef4444); color: #fff; \
                                font-size: 9px; font-weight: 700; border-radius: 999px; \
                                min-width: 14px; height: 14px; display: flex; \
                                align-items: center; justify-content: center; padding: 0 3px;",
                        "{notification_count}"
                    }
                }
            }

            // User avatar chip
            div {
                style: "display: flex; align-items: center; gap: 6px; \
                        padding: 4px 8px; border-radius: var(--fs-radius-sm); \
                        background: var(--fs-bg-elevated);",
                div {
                    style: "width: 24px; height: 24px; border-radius: 50%; flex-shrink: 0; \
                            background: var(--fs-primary); display: flex; \
                            align-items: center; justify-content: center; \
                            font-size: 11px; font-weight: 600; color: #fff;",
                    "{initial}"
                }
                span {
                    style: "font-size: 12px; color: var(--fs-text-primary); \
                            max-width: 80px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                    "{user_name}"
                }
            }
        }
    }
}

// ── Taskbar body ──────────────────────────────────────────────────────────────

#[component]
fn WebTaskbarBody(apps: Vec<AppEntry>, on_launch: Option<EventHandler<String>>) -> Element {
    rsx! {
        div {
            class: "fs-web-taskbar__body",
            style: "height: 48px; background: var(--fs-bg-sidebar, #0a0f1a); \
                    border-top: 1px solid var(--fs-border); \
                    display: flex; align-items: center; gap: 4px; padding: 0 8px; \
                    overflow-x: auto; overflow-y: hidden;",

            for app in &apps {
                button {
                    key: "{app.id}",
                    title: "{app.label_key}",
                    style: "display: flex; flex-direction: column; align-items: center; justify-content: center; \
                            gap: 2px; background: none; border: none; cursor: pointer; \
                            padding: 4px 8px; border-radius: var(--fs-radius-sm); flex-shrink: 0; \
                            color: var(--fs-text-primary);",
                    onclick: {
                        let id = app.id.clone();
                        let handler = on_launch;
                        move |_| {
                            if let Some(h) = &handler { h.call(id.clone()); }
                        }
                    },
                    span { style: "font-size: 20px; line-height: 1;", "{app.icon}" }
                    if app.is_running() {
                        div {
                            style: "width: 4px; height: 4px; border-radius: 50%; \
                                    background: var(--fs-primary);",
                        }
                    }
                }
            }
        }
    }
}
