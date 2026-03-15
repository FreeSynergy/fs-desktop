/// Desktop settings — taskbar position, display mode, autostart.
use std::path::PathBuf;

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config_path;

// ── TaskbarPosition ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskbarPosition {
    #[default]
    Bottom,
    Top,
    Left,
    Right,
}

impl TaskbarPosition {
    pub fn label(&self) -> &str {
        match self {
            Self::Bottom => "Bottom",
            Self::Top    => "Top",
            Self::Left   => "Left",
            Self::Right  => "Right",
        }
    }
}

// ── DisplayMode ────────────────────────────────────────────────────────────────

/// Preferred rendering mode for the desktop.
/// Saved to `~/.config/fsn/desktop.toml` and applied on the next launch.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DisplayMode {
    /// Native OS window (Dioxus desktop).
    #[default]
    Window,
    /// Browser / web server (Dioxus web).
    Web,
    /// Terminal UI (Dioxus TUI).
    Tui,
}

impl DisplayMode {
    pub fn label(&self) -> &str {
        match self {
            Self::Window => "Window",
            Self::Web    => "Web",
            Self::Tui    => "TUI",
        }
    }

    pub fn description(&self) -> &str {
        match self {
            Self::Window => "Native OS window (default)",
            Self::Web    => "Browser / web server",
            Self::Tui    => "Terminal UI",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Window => "🖥",
            Self::Web    => "🌐",
            Self::Tui    => "⬛",
        }
    }
}

// ── SidebarPosition ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SidebarPosition {
    #[default]
    Left,
    Right,
    Top,
    Bottom,
}

impl SidebarPosition {
    pub fn label(&self) -> &str {
        match self {
            Self::Left   => "Left",
            Self::Right  => "Right",
            Self::Top    => "Top",
            Self::Bottom => "Bottom",
        }
    }
}

// ── SidebarConfig ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SidebarConfig {
    #[serde(default)]
    pub position: SidebarPosition,
    #[serde(default = "default_true")]
    pub collapsible: bool,
    #[serde(default)]
    pub default_collapsed: bool,
    #[serde(default = "default_sidebar_width")]
    pub width: u32,
}

fn default_true() -> bool { true }
fn default_sidebar_width() -> u32 { 240 }

impl Default for SidebarConfig {
    fn default() -> Self {
        Self {
            position: SidebarPosition::Left,
            collapsible: true,
            default_collapsed: false,
            width: 240,
        }
    }
}

// ── DesktopConfig ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct DesktopConfig {
    #[serde(default)]
    pub taskbar_pos:  TaskbarPosition,
    #[serde(default)]
    pub display_mode: DisplayMode,
    #[serde(default)]
    pub sidebar: SidebarConfig,
}

impl DesktopConfig {
    pub fn load() -> Self {
        let path = desktop_toml_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) {
        let path = desktop_toml_path();
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Ok(text) = toml::to_string_pretty(self) {
            let _ = std::fs::write(&path, text);
        }
    }
}

fn desktop_toml_path() -> PathBuf {
    config_path("desktop.toml")
}

// ── DesktopSettings component ─────────────────────────────────────────────────

/// Desktop behavior settings component.
#[component]
pub fn DesktopSettings() -> Element {
    let config = use_signal(DesktopConfig::load);

    rsx! {
        div {
            class: "fsd-desktop-settings",
            style: "padding: 24px; max-width: 500px;",

            h3 { style: "margin-top: 0;", "Desktop" }

            // Display Mode
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Display Mode" }
                p { style: "font-size: 13px; color: var(--fsn-color-text-muted); margin: 0 0 8px;",
                    "Takes effect on the next launch."
                }
                div { style: "display: flex; flex-direction: column; gap: 6px;",
                    for mode in [DisplayMode::Window, DisplayMode::Web, DisplayMode::Tui] {
                        DisplayModeBtn { mode: mode.clone(), config }
                    }
                }
            }

            // Taskbar Position
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 8px;", "Taskbar Position" }
                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px;",
                    TaskbarPosBtn { pos: TaskbarPosition::Bottom, config }
                    TaskbarPosBtn { pos: TaskbarPosition::Top,    config }
                    TaskbarPosBtn { pos: TaskbarPosition::Left,   config }
                    TaskbarPosBtn { pos: TaskbarPosition::Right,  config }
                }
            }

            // Sidebar Position
            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 8px;", "Sidebar Position" }
                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px;",
                    SidebarPosBtn { pos: SidebarPosition::Left,   config }
                    SidebarPosBtn { pos: SidebarPosition::Right,  config }
                    SidebarPosBtn { pos: SidebarPosition::Top,    config }
                    SidebarPosBtn { pos: SidebarPosition::Bottom, config }
                }
            }

            // Sidebar collapse default
            SidebarCollapseToggle { config }

            button {
                style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                onclick: move |_| config.read().save(),
                "Save"
            }
        }
    }
}

#[component]
fn DisplayModeBtn(mode: DisplayMode, config: Signal<DesktopConfig>) -> Element {
    let is_active = config.read().display_mode == mode;
    let border = if is_active { "var(--fsn-color-primary)" } else { "var(--fsn-color-border-default)" };
    let weight = if is_active { "600" } else { "400" };
    rsx! {
        button {
            style: "display: flex; align-items: center; gap: 10px; padding: 10px 14px; \
                    border-radius: var(--fsn-radius-md); border: 2px solid {border}; \
                    cursor: pointer; background: var(--fsn-color-bg-surface); \
                    text-align: left; font-weight: {weight};",
            onclick: move |_| config.write().display_mode = mode.clone(),
            span { style: "font-size: 18px;", "{mode.icon()}" }
            div {
                span { style: "display: block; font-size: 14px;", "{mode.label()}" }
                span { style: "display: block; font-size: 12px; color: var(--fsn-color-text-muted);", "{mode.description()}" }
            }
        }
    }
}

#[component]
fn TaskbarPosBtn(pos: TaskbarPosition, config: Signal<DesktopConfig>) -> Element {
    let is_active = config.read().taskbar_pos == pos;
    let border = if is_active { "var(--fsn-color-primary)" } else { "var(--fsn-color-border-default)" };
    let label  = pos.label();
    rsx! {
        button {
            style: "padding: 10px; border-radius: var(--fsn-radius-md); border: 2px solid {border}; cursor: pointer; \
                    background: var(--fsn-color-bg-surface); color: var(--fsn-text-primary);",
            onclick: move |_| config.write().taskbar_pos = pos.clone(),
            "{label}"
        }
    }
}

#[component]
fn SidebarCollapseToggle(config: Signal<DesktopConfig>) -> Element {
    let checked = config.read().sidebar.default_collapsed;
    rsx! {
        div { style: "margin-bottom: 24px; display: flex; align-items: center; gap: 10px;",
            input {
                r#type: "checkbox",
                checked,
                onchange: move |evt| config.write().sidebar.default_collapsed = evt.checked(),
            }
            label { style: "font-size: 14px; color: var(--fsn-text-primary);",
                "Start with sidebar collapsed"
            }
        }
    }
}

#[component]
fn SidebarPosBtn(pos: SidebarPosition, config: Signal<DesktopConfig>) -> Element {
    let is_active = config.read().sidebar.position == pos;
    let border = if is_active { "var(--fsn-color-primary)" } else { "var(--fsn-color-border-default)" };
    let label  = pos.label();
    rsx! {
        button {
            style: "padding: 10px; border-radius: var(--fsn-radius-md); border: 2px solid {border}; cursor: pointer; \
                    background: var(--fsn-color-bg-surface); color: var(--fsn-text-primary);",
            onclick: move |_| config.write().sidebar.position = pos.clone(),
            "{label}"
        }
    }
}
