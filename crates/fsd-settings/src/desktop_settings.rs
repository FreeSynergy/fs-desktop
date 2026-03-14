/// Desktop settings — taskbar position, autostart, window behavior.
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

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

/// Desktop behavior settings component.
#[component]
pub fn DesktopSettings() -> Element {
    let taskbar_pos = use_signal(TaskbarPosition::default);
    let autostart: Signal<Vec<String>> = use_signal(Vec::new);

    rsx! {
        div {
            class: "fsd-desktop-settings",
            style: "padding: 24px; max-width: 500px;",

            h3 { style: "margin-top: 0;", "Desktop" }

            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 8px;", "Taskbar Position" }
                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 8px;",
                    for pos in [TaskbarPosition::Bottom, TaskbarPosition::Top, TaskbarPosition::Left, TaskbarPosition::Right] {
                        button {
                            style: "padding: 10px; border-radius: var(--fsn-radius-md); border: 2px solid {if *taskbar_pos.read() == pos { \"var(--fsn-color-primary)\" } else { \"var(--fsn-color-border-default)\" }}; cursor: pointer; background: var(--fsn-color-bg-surface);",
                            onclick: {
                                let pos = pos.clone();
                                move |_| *taskbar_pos.write() = pos.clone()
                            },
                            "{pos.label()}"
                        }
                    }
                }
            }

            div { style: "margin-bottom: 24px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 8px;", "Autostart Apps" }
                p { style: "font-size: 13px; color: var(--fsn-color-text-muted);",
                    "Apps in this list open automatically when the desktop starts."
                }
                // TODO: app picker for autostart
            }

            button {
                style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                "Save"
            }
        }
    }
}
