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

/// Global CSS: Midnight Blue theme variables + page-transition animations.
/// Injected at the root and within every AppShell so variables are always available.
pub const GLOBAL_CSS: &str = r#"
:root, [data-theme="midnight-blue"] {
    /* ── Midnight Blue – backgrounds ──────────────────────────────── */
    --fsn-bg-base:     #0c1222;
    --fsn-bg-surface:  #162032;
    --fsn-bg-elevated: #1e2d45;
    --fsn-bg-sidebar:  #0a0f1a;
    --fsn-bg-card:     #1a2538;
    --fsn-bg-input:    #0f1a2e;
    --fsn-bg-hover:    #243352;

    /* ── Text (WCAG AAA on #0c1222) ───────────────────────────────── */
    --fsn-text-primary:   #e8edf5;
    --fsn-text-secondary: #a0b0c8;
    --fsn-text-muted:     #5a6e88;
    --fsn-text-bright:    #ffffff;

    /* ── Primary – luminous blue ──────────────────────────────────── */
    --fsn-primary:       #4d8bf5;
    --fsn-primary-hover: #3a78e8;
    --fsn-primary-text:  #ffffff;
    --fsn-primary-glow:  rgba(77, 139, 245, 0.35);

    /* ── Accent – cyan ────────────────────────────────────────────── */
    --fsn-accent:       #22d3ee;
    --fsn-accent-hover: #06b6d4;

    /* ── Status ───────────────────────────────────────────────────── */
    --fsn-success:    #34d399;
    --fsn-success-bg: rgba(52, 211, 153, 0.12);
    --fsn-warning:    #fbbf24;
    --fsn-warning-bg: rgba(251, 191, 36, 0.12);
    --fsn-error:      #f87171;
    --fsn-error-bg:   rgba(248, 113, 113, 0.12);
    --fsn-info:       #60a5fa;

    /* ── Borders ──────────────────────────────────────────────────── */
    --fsn-border:       rgba(148, 170, 200, 0.18);
    --fsn-border-focus: #4d8bf5;
    --fsn-border-hover: rgba(148, 170, 200, 0.3);

    /* ── Sidebar ──────────────────────────────────────────────────── */
    --fsn-sidebar-text:      #a0b0c8;
    --fsn-sidebar-active:    #4d8bf5;
    --fsn-sidebar-active-bg: rgba(77, 139, 245, 0.15);
    --fsn-sidebar-hover-bg:  rgba(255, 255, 255, 0.05);

    /* ── Glassmorphism ────────────────────────────────────────────── */
    --fsn-glass-bg:   rgba(22, 32, 50, 0.75);
    --fsn-glass-border: rgba(148, 170, 200, 0.12);
    --fsn-glass-blur: 16px;

    /* ── Shadows ──────────────────────────────────────────────────── */
    --fsn-shadow:      0 4px 16px rgba(0, 0, 0, 0.4);
    --fsn-shadow-glow: 0 0 24px rgba(77, 139, 245, 0.2);

    /* ── Motion (--fsn-anim-duration: 0ms disables all animations) ─── */
    --fsn-anim-duration: 180ms;
    --fsn-transition: all var(--fsn-anim-duration) ease;

    /* ── Geometry ─────────────────────────────────────────────────── */
    --fsn-radius-sm: 6px;
    --fsn-radius-md: 10px;
    --fsn-radius-lg: 14px;

    /* ── Typography ───────────────────────────────────────────────── */
    --fsn-font:      'Inter', system-ui, sans-serif;
    --fsn-font-mono: 'JetBrains Mono', monospace;
    --fsn-font-size: 15px;

    /* ── Window frame (glassmorphism) ─────────────────────────────── */
    --fsn-window-bg:     rgba(15, 23, 42, 0.80);
    --fsn-window-border: rgba(255, 255, 255, 0.10);
    --fsn-window-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);

    /* ── Compat aliases for existing --fsn-color-* usage ─────────── */
    --fsn-color-primary:       var(--fsn-primary);
    --fsn-color-bg-base:       var(--fsn-bg-base);
    --fsn-color-bg-surface:    var(--fsn-bg-surface);
    --fsn-color-bg-sidebar:    var(--fsn-bg-sidebar);
    --fsn-color-bg-panel:      var(--fsn-bg-card);
    --fsn-color-bg-card:       var(--fsn-bg-card);
    --fsn-color-bg-overlay:    var(--fsn-bg-elevated);
    --fsn-color-bg-active:     var(--fsn-bg-elevated);
    --fsn-color-bg-input:      var(--fsn-bg-input);
    --fsn-color-text-primary:  var(--fsn-text-primary);
    --fsn-color-text-secondary: var(--fsn-text-secondary);
    --fsn-color-text-muted:    var(--fsn-text-muted);
    --fsn-color-text-inverse:  var(--fsn-text-primary);
    --fsn-color-border-default: var(--fsn-border);
    --fsn-color-success:       var(--fsn-success);
    --fsn-color-warning:       var(--fsn-warning);
    --fsn-color-error:         var(--fsn-error);
    --fsn-color-info:          var(--fsn-info);
}

* { box-sizing: border-box; margin: 0; padding: 0; }

html, body { height: 100%; overflow: hidden; }

body {
    background: var(--fsn-bg-base);
    color: var(--fsn-text-primary);
    font-family: var(--fsn-font);
    font-size: var(--fsn-font-size);
}

/* ── Page-transition animations ───────────────────────────────────── */
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

/* ── Window control buttons (KDE/Breeze style) ─────────────────── */
.fsd-window-btn {
    width: 22px; height: 20px;
    border-radius: var(--fsn-radius-sm);
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer; padding: 0;
    display: inline-flex; align-items: center; justify-content: center;
    color: var(--fsn-text-secondary);
    transition: background 120ms, border-color 120ms, color 120ms;
    flex-shrink: 0;
}
.fsd-window-btn:hover {
    background: var(--fsn-bg-hover);
    border-color: var(--fsn-border);
}
.fsd-window-btn--close:hover {
    background: var(--fsn-error-bg);
    border-color: var(--fsn-error);
    color: var(--fsn-error);
}

/* ── Disabled button state ─────────────────────────────────────── */
button:disabled,
.fsd-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
    pointer-events: none;
}

/* ── Scrollable container ──────────────────────────────────────── */
.fsn-scrollable {
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: var(--fsn-border) transparent;
}
.fsn-scrollable::-webkit-scrollbar { width: 6px; }
.fsn-scrollable::-webkit-scrollbar-track { background: transparent; }
.fsn-scrollable::-webkit-scrollbar-thumb {
    background: var(--fsn-border);
    border-radius: 3px;
}
.fsn-scrollable::-webkit-scrollbar-thumb:hover {
    background: var(--fsn-border-hover);
}

/* ── Content area ──────────────────────────────────────────────── */
.fsn-content { flex: 1; overflow-y: auto; }

/* ── Sidebar collapsed: CSS hover-tooltip via ::after ─────────── */
.fsd-sidebar--collapsed .fsd-sidebar__item {
    position: relative;
}
.fsd-sidebar--collapsed .fsd-sidebar__item::after {
    content: attr(data-label);
    position: absolute;
    left: calc(100% + 10px);
    top: 50%;
    transform: translateY(-50%);
    background: var(--fsn-bg-elevated);
    color: var(--fsn-text-primary);
    border: 1px solid var(--fsn-border);
    border-radius: var(--fsn-radius-md);
    padding: 4px 10px;
    font-size: 12px;
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transition: opacity 120ms ease;
    z-index: 9999;
    box-shadow: var(--fsn-shadow);
}
.fsd-sidebar--collapsed .fsd-sidebar__item:hover::after {
    opacity: 1;
}

/* ── Component styles: Window Chrome ──────────────────────────── */

/* macOS: circular colored dots */
[data-chrome-style="macos"] .fsd-window-btn {
    width: 12px; height: 12px;
    border-radius: 50%;
    background: var(--fsn-text-muted);
    border: none;
    padding: 0;
    transition: background 120ms;
}
[data-chrome-style="macos"] .fsd-window-btn:first-child  { background: #febc2e; }
[data-chrome-style="macos"] .fsd-window-btn:nth-child(2) { background: #28c840; }
[data-chrome-style="macos"] .fsd-window-btn--close       { background: #ff5f57; }
[data-chrome-style="macos"] .fsd-window-btn svg { display: none; }
[data-chrome-style="macos"] .fsd-window-btn:hover { opacity: 0.8; }

/* Windows: flat square buttons */
[data-chrome-style="windows"] .fsd-window-btn {
    width: 46px; height: 36px;
    border-radius: 0;
    border: none;
    background: transparent;
    color: var(--fsn-text-secondary);
    transition: background 80ms;
}
[data-chrome-style="windows"] .fsd-window-btn:hover { background: rgba(255,255,255,0.1); }
[data-chrome-style="windows"] .fsd-window-btn--close:hover { background: #e81123; color: #fff; }

/* Minimal: single small X */
[data-chrome-style="minimal"] .fsd-window-btn:not(.fsd-window-btn--close) { display: none; }
[data-chrome-style="minimal"] .fsd-window-btn--close {
    width: 16px; height: 16px;
    border-radius: 3px;
    background: transparent;
    border: none;
    opacity: 0.5;
    transition: opacity 120ms;
}
[data-chrome-style="minimal"] .fsd-window-btn--close:hover { opacity: 1; background: var(--fsn-error-bg); }

/* ── Component styles: Button radius ──────────────────────────── */
[data-btn-style="square"]  { --fsn-radius-sm: 2px;   --fsn-radius-md: 2px;   --fsn-radius-lg: 4px;   }
[data-btn-style="pill"]    { --fsn-radius-sm: 999px; --fsn-radius-md: 999px; --fsn-radius-lg: 999px; }
[data-btn-style="flat"]    { --fsn-radius-sm: 0px;   --fsn-radius-md: 0px;   --fsn-radius-lg: 0px;   }
/* rounded is the default, no override needed */

/* ── Component styles: Sidebar ────────────────────────────────── */
[data-sidebar-style="glass"] .fsd-shell-sidebar {
    background: var(--fsn-glass-bg) !important;
    backdrop-filter: blur(var(--fsn-glass-blur, 16px));
    -webkit-backdrop-filter: blur(var(--fsn-glass-blur, 16px));
}
[data-sidebar-style="transparent"] .fsd-shell-sidebar {
    background: transparent !important;
}
"#;

/// Root app wrapper. Injects global CSS and applies mode-specific root styles.
#[component]
pub fn AppShell(mode: AppMode, children: Element) -> Element {
    let height = match mode {
        AppMode::Window     => "height: 100%; width: 100%;",
        AppMode::Standalone => "height: 100vh; width: 100vw;",
        AppMode::Tui        => "height: 100%; width: 100%;",
    };
    rsx! {
        style { "{GLOBAL_CSS}" }
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
                class: "fsd-layout-b__detail fsn-scrollable",
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
