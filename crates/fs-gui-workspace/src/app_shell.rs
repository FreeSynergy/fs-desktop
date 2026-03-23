/// AppShell — unified app wrapper with mode context and layout primitives.
use dioxus::prelude::*;

/// How an fs-* app is rendered.
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
    --fs-bg-base:     #0c1222;
    --fs-bg-surface:  #162032;
    --fs-bg-elevated: #1e2d45;
    --fs-bg-sidebar:  #0a0f1a;
    --fs-bg-card:     #1a2538;
    --fs-bg-input:    #0f1a2e;
    --fs-bg-hover:    #243352;

    /* ── Text (WCAG AAA on #0c1222) ───────────────────────────────── */
    --fs-text-primary:   #e8edf5;
    --fs-text-secondary: #a0b0c8;
    --fs-text-muted:     #5a6e88;
    --fs-text-bright:    #ffffff;

    /* ── Primary – luminous blue ──────────────────────────────────── */
    --fs-primary:       #4d8bf5;
    --fs-primary-hover: #3a78e8;
    --fs-primary-text:  #ffffff;
    --fs-primary-glow:  rgba(77, 139, 245, 0.35);

    /* ── Accent – cyan ────────────────────────────────────────────── */
    --fs-accent:       #22d3ee;
    --fs-accent-hover: #06b6d4;

    /* ── Status ───────────────────────────────────────────────────── */
    --fs-success:    #34d399;
    --fs-success-bg: rgba(52, 211, 153, 0.12);
    --fs-warning:    #fbbf24;
    --fs-warning-bg: rgba(251, 191, 36, 0.12);
    --fs-error:      #f87171;
    --fs-error-bg:   rgba(248, 113, 113, 0.12);
    --fs-info:       #60a5fa;

    /* ── Borders ──────────────────────────────────────────────────── */
    --fs-border:       rgba(148, 170, 200, 0.18);
    --fs-border-focus: #4d8bf5;
    --fs-border-hover: rgba(148, 170, 200, 0.3);

    /* ── Sidebar ──────────────────────────────────────────────────── */
    --fs-sidebar-text:      #a0b0c8;
    --fs-sidebar-active:    #4d8bf5;
    --fs-sidebar-active-bg: rgba(77, 139, 245, 0.15);
    --fs-sidebar-hover-bg:  rgba(255, 255, 255, 0.05);

    /* ── Glassmorphism ────────────────────────────────────────────── */
    --fs-glass-bg:   rgba(22, 32, 50, 0.75);
    --fs-glass-border: rgba(148, 170, 200, 0.12);
    --fs-glass-blur: 16px;

    /* ── Shadows ──────────────────────────────────────────────────── */
    --fs-shadow:      0 4px 16px rgba(0, 0, 0, 0.4);
    --fs-shadow-glow: 0 0 24px rgba(77, 139, 245, 0.2);

    /* ── Motion (--fs-anim-duration: 0ms disables all animations) ─── */
    --fs-anim-duration: 180ms;
    --fs-transition: all var(--fs-anim-duration) ease;

    /* ── Geometry ─────────────────────────────────────────────────── */
    --fs-radius-sm: 6px;
    --fs-radius-md: 10px;
    --fs-radius-lg: 14px;

    /* ── Typography ───────────────────────────────────────────────── */
    --fs-font:      'Inter', system-ui, sans-serif;
    --fs-font-mono: 'JetBrains Mono', monospace;
    --fs-font-size: 15px;

    /* ── Window frame (glassmorphism) ─────────────────────────────── */
    --fs-window-bg:     rgba(15, 23, 42, 0.80);
    --fs-window-border: rgba(255, 255, 255, 0.10);
    --fs-window-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);

    /* ── Compat aliases for existing --fs-color-* usage ─────────── */
    --fs-color-primary:       var(--fs-primary);
    --fs-color-bg-base:       var(--fs-bg-base);
    --fs-color-bg-surface:    var(--fs-bg-surface);
    --fs-color-bg-sidebar:    var(--fs-bg-sidebar);
    --fs-color-bg-panel:      var(--fs-bg-card);
    --fs-color-bg-card:       var(--fs-bg-card);
    --fs-color-bg-overlay:    var(--fs-bg-elevated);
    --fs-color-bg-active:     var(--fs-bg-elevated);
    --fs-color-bg-input:      var(--fs-bg-input);
    --fs-color-text-primary:  var(--fs-text-primary);
    --fs-color-text-secondary: var(--fs-text-secondary);
    --fs-color-text-muted:    var(--fs-text-muted);
    --fs-color-text-inverse:  var(--fs-text-primary);
    --fs-color-border-default: var(--fs-border);
    --fs-color-success:       var(--fs-success);
    --fs-color-warning:       var(--fs-warning);
    --fs-color-error:         var(--fs-error);
    --fs-color-info:          var(--fs-info);
}

* { box-sizing: border-box; margin: 0; padding: 0; }

html, body { height: 100%; overflow: hidden; }

body {
    background: var(--fs-bg-base);
    color: var(--fs-text-primary);
    font-family: var(--fs-font);
    font-size: var(--fs-font-size);
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
.fs-page-enter { animation: slideInRight 180ms ease forwards; }
.fs-page-fade  { animation: fadeInUp 140ms ease forwards; }
@media (prefers-reduced-motion: reduce) {
    .fs-page-enter, .fs-page-fade { animation: none; }
}

/* ── Window control buttons (KDE/Breeze style) ─────────────────── */
.fs-window-btn {
    width: 22px; height: 20px;
    border-radius: var(--fs-radius-sm);
    background: transparent;
    border: 1px solid transparent;
    cursor: pointer; padding: 0;
    display: inline-flex; align-items: center; justify-content: center;
    color: var(--fs-text-secondary);
    transition: background 120ms, border-color 120ms, color 120ms;
    flex-shrink: 0;
}
.fs-window-btn:hover {
    background: var(--fs-bg-hover);
    border-color: var(--fs-border);
}
.fs-window-btn--close:hover {
    background: var(--fs-error-bg);
    border-color: var(--fs-error);
    color: var(--fs-error);
}

/* ── Disabled button state ─────────────────────────────────────── */
button:disabled,
.fs-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
    pointer-events: none;
}

/* ── Scrollable container ──────────────────────────────────────── */
.fs-scrollable {
    overflow-y: auto;
    overflow-x: hidden;
    scrollbar-width: thin;
    scrollbar-color: var(--fs-border) transparent;
}
.fs-scrollable::-webkit-scrollbar { width: 6px; }
.fs-scrollable::-webkit-scrollbar-track { background: transparent; }
.fs-scrollable::-webkit-scrollbar-thumb {
    background: var(--fs-border);
    border-radius: 3px;
}
.fs-scrollable::-webkit-scrollbar-thumb:hover {
    background: var(--fs-border-hover);
}

/* ── Content area ──────────────────────────────────────────────── */
.fs-content { flex: 1; overflow-y: auto; }

/* ── Sidebar collapsed: CSS hover-tooltip via ::after ─────────── */
.fs-sidebar--collapsed .fs-sidebar__item {
    position: relative;
}
.fs-sidebar--collapsed .fs-sidebar__item::after {
    content: attr(data-label);
    position: absolute;
    left: calc(100% + 10px);
    top: 50%;
    transform: translateY(-50%);
    background: var(--fs-bg-elevated);
    color: var(--fs-text-primary);
    border: 1px solid var(--fs-border);
    border-radius: var(--fs-radius-md);
    padding: 4px 10px;
    font-size: 12px;
    white-space: nowrap;
    pointer-events: none;
    opacity: 0;
    transition: opacity 120ms ease;
    z-index: 9999;
    box-shadow: var(--fs-shadow);
}
.fs-sidebar--collapsed .fs-sidebar__item:hover::after {
    opacity: 1;
}

/* ── Component styles: Window Chrome ──────────────────────────── */

/* macOS: circular colored dots */
[data-chrome-style="macos"] .fs-window-btn {
    width: 12px; height: 12px;
    border-radius: 50%;
    background: var(--fs-text-muted);
    border: none;
    padding: 0;
    transition: background 120ms;
}
[data-chrome-style="macos"] .fs-window-btn:first-child  { background: #febc2e; }
[data-chrome-style="macos"] .fs-window-btn:nth-child(2) { background: #28c840; }
[data-chrome-style="macos"] .fs-window-btn--close       { background: #ff5f57; }
[data-chrome-style="macos"] .fs-window-btn svg { display: none; }
[data-chrome-style="macos"] .fs-window-btn:hover { opacity: 0.8; }

/* Windows: flat square buttons */
[data-chrome-style="windows"] .fs-window-btn {
    width: 46px; height: 36px;
    border-radius: 0;
    border: none;
    background: transparent;
    color: var(--fs-text-secondary);
    transition: background 80ms;
}
[data-chrome-style="windows"] .fs-window-btn:hover { background: rgba(255,255,255,0.1); }
[data-chrome-style="windows"] .fs-window-btn--close:hover { background: #e81123; color: #fff; }

/* Minimal: single small X */
[data-chrome-style="minimal"] .fs-window-btn:not(.fs-window-btn--close) { display: none; }
[data-chrome-style="minimal"] .fs-window-btn--close {
    width: 16px; height: 16px;
    border-radius: 3px;
    background: transparent;
    border: none;
    opacity: 0.5;
    transition: opacity 120ms;
}
[data-chrome-style="minimal"] .fs-window-btn--close:hover { opacity: 1; background: var(--fs-error-bg); }

/* ── Component styles: Button radius ──────────────────────────── */
[data-btn-style="square"]  { --fs-radius-sm: 2px;   --fs-radius-md: 2px;   --fs-radius-lg: 4px;   }
[data-btn-style="pill"]    { --fs-radius-sm: 999px; --fs-radius-md: 999px; --fs-radius-lg: 999px; }
[data-btn-style="flat"]    { --fs-radius-sm: 0px;   --fs-radius-md: 0px;   --fs-radius-lg: 0px;   }
/* rounded is the default, no override needed */

/* ── Component styles: Sidebar ────────────────────────────────── */
[data-sidebar-style="glass"] .fs-workspace-sidebar {
    background: var(--fs-glass-bg) !important;
    backdrop-filter: blur(var(--fs-glass-blur, 16px));
    -webkit-backdrop-filter: blur(var(--fs-glass-blur, 16px));
}
[data-sidebar-style="transparent"] .fs-workspace-sidebar {
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
            class: "fs-app-shell",
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
            class: "fs-screen-wrapper",
            style: "flex: 1; overflow: {overflow}; padding: {padding}; max-width: {max_w}; \
                    width: 100%; box-sizing: border-box;",
            {children}
        }
    }
}

// ── Standard Layouts ──────────────────────────────────────────────────────────

/// Layout A — full-width scrollable column (fs-store, fs-builder).
#[component]
pub fn LayoutA(children: Element) -> Element {
    rsx! {
        div {
            class: "fs-layout-a fs-page-enter",
            style: "display: flex; flex-direction: column; height: 100%; width: 100%; overflow: hidden;",
            {children}
        }
    }
}

/// Layout B — fixed sidebar (master) + scrollable detail pane.
/// Used for: fs-container-app, fs-settings.
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
            class: "fs-layout-b fs-page-enter",
            style: "display: flex; height: 100%; width: 100%; overflow: hidden;",
            div {
                class: "fs-layout-b__master",
                style: "width: {props.sidebar_width}px; flex-shrink: 0; overflow-y: auto; \
                        background: var(--fs-color-bg-surface, #0f172a); \
                        border-right: 1px solid var(--fs-color-border-default, #334155);",
                {props.master}
            }
            div {
                class: "fs-layout-b__detail fs-scrollable",
                style: "flex: 1; overflow: auto;",
                {props.children}
            }
        }
    }
}

/// Layout C — centered card (fs-profile, login screens).
#[component]
pub fn LayoutC(
    #[props(default = 640)]
    max_width: u32,
    children: Element,
) -> Element {
    rsx! {
        div {
            class: "fs-layout-c fs-page-fade",
            style: "display: flex; justify-content: center; overflow: auto; \
                    height: 100%; width: 100%; padding: 32px 24px; box-sizing: border-box;",
            div {
                style: "width: 100%; max-width: {max_width}px;",
                {children}
            }
        }
    }
}
