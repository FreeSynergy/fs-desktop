/// AppShell — unified app wrapper with mode context and layout primitives.
use dioxus::prelude::*;

/// How an fsd-* app is rendered.
#[derive(Clone, PartialEq, Debug, Default)]
pub enum AppMode {
    /// Embedded inside a WindowFrame in the desktop shell.
    #[default]
    Window,
    /// Running as its own top-level OS window.
    Standalone,
    /// Running inside a terminal (dioxus-terminal).
    Tui,
}

/// Shared CSS for page-transition animations.
const TRANSITION_CSS: &str = r#"
@keyframes slideInRight {
    from { opacity: 0; transform: translateX(12px); }
    to   { opacity: 1; transform: translateX(0); }
}
@keyframes fadeInUp {
    from { opacity: 0; transform: translateY(6px); }
    to   { opacity: 1; transform: translateY(0); }
}
.fsd-page-enter { animation: slideInRight 180ms ease forwards; }
.fsd-page-fade  { animation: fadeInUp 140ms ease forwards; }
@media (prefers-reduced-motion: reduce) {
    .fsd-page-enter, .fsd-page-fade { animation: none; }
}
"#;

/// Root app wrapper. Injects transition CSS and applies mode-specific root styles.
#[component]
pub fn AppShell(mode: AppMode, children: Element) -> Element {
    let height = match mode {
        AppMode::Window     => "height: 100%; width: 100%;",
        AppMode::Standalone => "height: 100vh; width: 100vw;",
        AppMode::Tui        => "height: 100%; width: 100%;",
    };
    rsx! {
        style { "{TRANSITION_CSS}" }
        div {
            class: "fsd-app-shell",
            style: "display: flex; flex-direction: column; {height} overflow: hidden;",
            {children}
        }
    }
}

/// Consistent content wrapper: max-width, padding, and scroll behavior.
#[component]
pub fn ScreenWrapper(
    max_width: Option<String>,
    #[props(default = true)]
    scroll: bool,
    #[props(default = "24px".to_string())]
    padding: String,
    children: Element,
) -> Element {
    let overflow = if scroll { "auto" } else { "hidden" };
    let max_w    = max_width.as_deref().unwrap_or("none");
    rsx! {
        div {
            class: "fsd-screen-wrapper",
            style: "flex: 1; overflow: {overflow}; padding: {padding}; max-width: {max_w}; \
                    width: 100%; box-sizing: border-box;",
            {children}
        }
    }
}

// ── Standard Layouts ──────────────────────────────────────────────────────────

/// Layout A — full-width scrollable column (fsd-store, fsd-studio).
#[component]
pub fn LayoutA(children: Element) -> Element {
    rsx! {
        div {
            class: "fsd-layout-a fsd-page-enter",
            style: "display: flex; flex-direction: column; height: 100%; width: 100%; overflow: hidden;",
            {children}
        }
    }
}

/// Layout B — fixed sidebar (master) + scrollable detail pane.
/// Used for: fsd-conductor, fsd-settings.
#[derive(Props, Clone, PartialEq)]
pub struct LayoutBProps {
    #[props(default = 240)]
    pub sidebar_width: u32,
    pub master: Element,
    pub children: Element,
}

#[component]
pub fn LayoutB(props: LayoutBProps) -> Element {
    rsx! {
        div {
            class: "fsd-layout-b fsd-page-enter",
            style: "display: flex; height: 100%; width: 100%; overflow: hidden;",
            div {
                class: "fsd-layout-b__master",
                style: "width: {props.sidebar_width}px; flex-shrink: 0; overflow-y: auto; \
                        background: var(--fsn-color-bg-surface, #0f172a); \
                        border-right: 1px solid var(--fsn-color-border-default, #334155);",
                {props.master}
            }
            div {
                class: "fsd-layout-b__detail",
                style: "flex: 1; overflow: auto;",
                {props.children}
            }
        }
    }
}

/// Layout C — centered card (fsd-profile, login screens).
#[component]
pub fn LayoutC(
    #[props(default = 640)]
    max_width: u32,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "fsd-layout-c fsd-page-fade",
            style: "display: flex; justify-content: center; overflow: auto; \
                    height: 100%; width: 100%; padding: 32px 24px; box-sizing: border-box;",
            div {
                style: "width: 100%; max-width: {max_width}px;",
                {children}
            }
        }
    }
}
