/// Keyboard shortcuts — action registry, persistence, and settings UI.
use dioxus::prelude::*;
use fs_i18n;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::config_path;

/// A registered desktop action with an optional keyboard shortcut.
#[derive(Clone, PartialEq, Debug)]
pub struct ActionDef {
    pub id: &'static str,
    pub label: String,
    pub category: &'static str,
    pub default_shortcut: Option<&'static str>,
}

/// All desktop actions. The source of truth for shortcut defaults.
pub fn register_actions() -> Vec<ActionDef> {
    vec![
        ActionDef { id: "app.settings",      label: fs_i18n::t("settings.shortcuts.action_open_settings"), category: "App",    default_shortcut: Some("Ctrl+,") },
        ActionDef { id: "app.launcher",       label: fs_i18n::t("settings.shortcuts.action_launcher"),      category: "App",    default_shortcut: Some("Ctrl+Space") },
        ActionDef { id: "app.quit",           label: fs_i18n::t("settings.shortcuts.action_quit"),          category: "App",    default_shortcut: Some("Ctrl+Q") },
        ActionDef { id: "view.fullscreen",    label: fs_i18n::t("settings.shortcuts.action_fullscreen"),    category: "View",   default_shortcut: Some("F11") },
        ActionDef { id: "view.sidebar.show",  label: fs_i18n::t("settings.shortcuts.action_sidebar"),       category: "View",   default_shortcut: None },
        ActionDef { id: "store.open",         label: fs_i18n::t("settings.shortcuts.action_store"),         category: "Tools",  default_shortcut: Some("Ctrl+S") },
        ActionDef { id: "store.install",      label: fs_i18n::t("settings.shortcuts.action_install"),       category: "Tools",  default_shortcut: Some("Ctrl+I") },
        ActionDef { id: "tasks.open",         label: fs_i18n::t("settings.shortcuts.action_tasks"),         category: "Tools",  default_shortcut: Some("Ctrl+T") },
        ActionDef { id: "container-app.open", label: fs_i18n::t("settings.shortcuts.action_container"), category: "Tools",  default_shortcut: None },
        ActionDef { id: "studio.open",        label: fs_i18n::t("settings.shortcuts.action_studio"),        category: "Tools",  default_shortcut: None },
        ActionDef { id: "bots.open",          label: fs_i18n::t("settings.shortcuts.action_bots"),          category: "Tools",  default_shortcut: None },
        ActionDef { id: "help.open",          label: fs_i18n::t("settings.shortcuts.action_help"),          category: "Help",   default_shortcut: Some("F1") },
        ActionDef { id: "help.shortcuts",     label: fs_i18n::t("settings.shortcuts.action_shortcuts"),     category: "Help",   default_shortcut: None },
        ActionDef { id: "window.close",       label: fs_i18n::t("settings.shortcuts.action_close"),         category: "Window", default_shortcut: Some("Escape") },
    ]
}

/// Returns the current shortcut for an action (custom > default).
pub fn resolve_shortcut<'a>(action: &'a ActionDef, config: &'a ShortcutsConfig) -> Option<&'a str> {
    config.custom
        .get(action.id)
        .map(|s| s.as_str())
        .or(action.default_shortcut)
}

/// Persisted custom shortcut overrides.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct ShortcutsConfig {
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

impl ShortcutsConfig {
    pub fn load() -> Self {
        let path = config_path("shortcuts.toml");
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self) {
        if let Ok(content) = toml::to_string(self) {
            let path = config_path("shortcuts.toml");
            let _ = std::fs::write(path, content);
        }
    }
}

// ── ShortcutsSettings ─────────────────────────────────────────────────────────

/// Keyboard shortcuts editor in Settings.
///
/// Groups all registered actions by category. Each row shows the action name
/// and a clickable shortcut field. Clicking enters "recording" mode — the next
/// key combo pressed becomes the new shortcut. Escape cancels recording.
#[component]
pub fn ShortcutsSettings() -> Element {
    let mut config = use_signal(ShortcutsConfig::load);
    let mut recording: Signal<Option<String>> = use_signal(|| None);
    let mut search = use_signal(String::new);

    let actions = register_actions();

    // Build sorted, deduplicated category list
    let mut categories: Vec<&str> = actions.iter().map(|a| a.category).collect();
    categories.sort();
    categories.dedup();

    let q = search.read().to_lowercase();
    let filtered: Vec<&ActionDef> = actions
        .iter()
        .filter(|a| q.is_empty() || a.label.to_lowercase().contains(&q) || a.category.to_lowercase().contains(&q))
        .collect();

    // Key recording handler — captures the next key combo when in recording mode
    let on_key = move |evt: KeyboardEvent| {
        if recording.read().is_none() {
            return;
        }
        evt.stop_propagation();

        let key = evt.key();
        // Ignore standalone modifier presses
        let key_str = match &key {
            Key::Character(c)  => c.to_uppercase(),
            Key::F1            => "F1".to_string(),
            Key::F2            => "F2".to_string(),
            Key::F3            => "F3".to_string(),
            Key::F4            => "F4".to_string(),
            Key::F5            => "F5".to_string(),
            Key::F6            => "F6".to_string(),
            Key::F7            => "F7".to_string(),
            Key::F8            => "F8".to_string(),
            Key::F9            => "F9".to_string(),
            Key::F10           => "F10".to_string(),
            Key::F11           => "F11".to_string(),
            Key::F12           => "F12".to_string(),
            Key::Tab           => "Tab".to_string(),
            Key::Enter         => "Enter".to_string(),
            Key::Delete        => "Delete".to_string(),
            Key::Backspace     => "Backspace".to_string(),
            Key::ArrowUp       => "Up".to_string(),
            Key::ArrowDown     => "Down".to_string(),
            Key::ArrowLeft     => "Left".to_string(),
            Key::ArrowRight    => "Right".to_string(),
            Key::Escape        => {
                // Escape cancels recording
                recording.set(None);
                return;
            }
            Key::Control | Key::Alt | Key::Shift | Key::Super | Key::Meta => return,
            _ => return,
        };

        let mods = evt.modifiers();
        let mut parts: Vec<&str> = Vec::new();
        if mods.ctrl()  { parts.push("Ctrl"); }
        if mods.alt()   { parts.push("Alt"); }
        if mods.shift() { parts.push("Shift"); }

        let shortcut = if parts.is_empty() {
            key_str.clone()
        } else {
            format!("{}+{}", parts.join("+"), key_str)
        };

        if let Some(action_id) = recording.read().clone() {
            config.write().custom.insert(action_id, shortcut);
            config.read().save();
        }
        recording.set(None);
    };

    rsx! {
        div {
            class: "fs-shortcuts",
            style: "padding: 24px; max-width: 640px; outline: none;",
            tabindex: "0",
            onkeydown: on_key,

            h3 { style: "margin-top: 0;", {fs_i18n::t("settings.shortcuts.title")} }

            // Search
            div { style: "margin-bottom: 20px;",
                input {
                    r#type: "search",
                    placeholder: fs_i18n::t("settings.shortcuts.search_placeholder"),
                    style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fs-border); \
                            border-radius: var(--fs-radius-md); font-size: 13px; \
                            background: var(--fs-bg-input); color: var(--fs-text-primary);",
                    oninput: move |e| *search.write() = e.value(),
                }
            }

            // Action groups
            for cat in &categories {
                {
                    let cat_actions: Vec<&&ActionDef> = filtered.iter().filter(|a| a.category == *cat).collect();
                    if cat_actions.is_empty() {
                        rsx! {}
                    } else {
                        let cfg = config.read().clone();
                        let recording_id = recording.read().clone();
                        rsx! {
                            div { style: "margin-bottom: 20px;",
                                div {
                                    style: "font-size: 11px; font-weight: 600; text-transform: uppercase; \
                                            letter-spacing: 0.08em; color: var(--fs-text-muted); \
                                            margin-bottom: 6px; padding-bottom: 4px; \
                                            border-bottom: 1px solid var(--fs-border);",
                                    "{cat}"
                                }
                                for action in cat_actions {
                                    {
                                        let action = *action;
                                        let current = resolve_shortcut(action, &cfg)
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| "—".to_string());
                                        let is_default = cfg.custom.get(action.id).is_none();
                                        let is_recording = recording_id.as_deref() == Some(action.id);
                                        let action_id = action.id.to_string();
                                        let action_id2 = action.id.to_string();
                                        rsx! {
                                            div {
                                                key: "{action.id}",
                                                style: "display: flex; align-items: center; \
                                                        justify-content: space-between; \
                                                        padding: 6px 0; gap: 12px;",
                                                span {
                                                    style: "font-size: 13px; color: var(--fs-text-primary); flex: 1;",
                                                    "{action.label}"
                                                }
                                                div { style: "display: flex; align-items: center; gap: 6px;",
                                                    // Shortcut badge (clickable to start recording)
                                                    button {
                                                        style: if is_recording {
                                                            "min-width: 120px; padding: 3px 10px; font-size: 12px; \
                                                             border-radius: var(--fs-radius-sm); cursor: pointer; \
                                                             background: var(--fs-primary); color: var(--fs-primary-text); \
                                                             border: 1px solid var(--fs-primary); font-family: var(--fs-font-mono);"
                                                        } else {
                                                            "min-width: 120px; padding: 3px 10px; font-size: 12px; \
                                                             border-radius: var(--fs-radius-sm); cursor: pointer; \
                                                             background: var(--fs-bg-elevated); color: var(--fs-text-secondary); \
                                                             border: 1px solid var(--fs-border); font-family: var(--fs-font-mono);"
                                                        },
                                                        onclick: move |_| {
                                                            if recording.read().as_deref() == Some(action_id.as_str()) {
                                                                recording.set(None);
                                                            } else {
                                                                recording.set(Some(action_id.clone()));
                                                            }
                                                        },
                                                        { if is_recording { fs_i18n::t("settings.shortcuts.press_keys") } else { current.clone() } }
                                                    }
                                                    // Reset button (only when customized)
                                                    if !is_default {
                                                        button {
                                                            style: "padding: 3px 8px; font-size: 11px; border-radius: var(--fs-radius-sm); \
                                                                    cursor: pointer; background: none; color: var(--fs-text-muted); \
                                                                    border: 1px solid var(--fs-border);",
                                                            onclick: move |_| {
                                                                config.write().custom.remove(&action_id2);
                                                                config.read().save();
                                                            },
                                                            {fs_i18n::t("actions.reset")}
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
                }
            }

            if *recording.read() != None {
                div {
                    style: "position: fixed; bottom: 20px; left: 50%; transform: translateX(-50%); \
                            background: var(--fs-bg-elevated); border: 1px solid var(--fs-primary); \
                            border-radius: var(--fs-radius-md); padding: 8px 20px; font-size: 13px; \
                            color: var(--fs-text-secondary); z-index: 9999;",
                    {fs_i18n::t("settings.shortcuts.recording_hint")}
                }
            }
        }
    }
}
