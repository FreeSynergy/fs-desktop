// form.rs — AddBotForm state and the "New Bot" form component.

use dioxus::prelude::*;

use super::model::{Bot, BotAction, BotTrigger};
use super::save_bots;

// ── AddBotForm ────────────────────────────────────────────────────────────────

/// Transient form state for the add-bot dialog.
#[derive(Clone, Default)]
pub struct AddBotForm {
    pub name:           String,
    pub description:    String,
    pub trigger_kind:   String, // "startup" | "interval"
    pub interval_secs:  String,
    pub action_kind:    String, // "start" | "stop" | "restart" | "command"
    pub service_or_cmd: String,
}

impl AddBotForm {
    pub fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
            && !self.service_or_cmd.trim().is_empty()
            && !self.action_kind.is_empty()
            && !self.trigger_kind.is_empty()
    }

    pub fn build_bot(&self) -> Option<Bot> {
        if !self.is_valid() {
            return None;
        }
        let trigger = match self.trigger_kind.as_str() {
            "interval" => {
                let secs = self.interval_secs.parse::<u64>().unwrap_or(300);
                BotTrigger::Interval { interval_secs: secs }
            }
            _ => BotTrigger::OnStartup,
        };
        let svc = self.service_or_cmd.trim().to_string();
        let action = match self.action_kind.as_str() {
            "stop"    => BotAction::Stop { service: svc },
            "restart" => BotAction::Restart { service: svc },
            "command" => BotAction::RunCommand { command: svc },
            _         => BotAction::Start { service: svc },
        };
        Some(Bot {
            name:        self.name.trim().to_string(),
            description: self.description.trim().to_string(),
            trigger,
            action,
            enabled: true,
        })
    }
}

// ── AddBotFormView ────────────────────────────────────────────────────────────

/// "New Bot" form panel — rendered when `show_add` is true.
#[component]
pub fn AddBotFormView(
    mut form: Signal<AddBotForm>,
    mut bots: Signal<Vec<Bot>>,
    mut show_add: Signal<bool>,
    mut status_msg: Signal<Option<String>>,
) -> Element {
    let show_interval = form.read().trigger_kind == "interval";
    let is_command    = form.read().action_kind == "command";
    let form_valid    = form.read().is_valid();
    let svc_label     = if is_command { "Command" } else { "Service name" };
    let svc_hint      = if is_command { "e.g. /usr/bin/fsn sync" } else { "e.g. zentinel" };

    rsx! {
        div {
            style: "padding: 16px; background: var(--fsn-color-bg-surface); \
                    border-radius: var(--fsn-radius-md); border: 1px solid var(--fsn-color-border-default); \
                    margin-bottom: 16px;",

            h4 { style: "margin: 0 0 12px;", "New Bot" }

            // Row 1: Name + Description
            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 12px;",
                div {
                    label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Name" }
                    input {
                        r#type: "text",
                        placeholder: "e.g. auto-restart-proxy",
                        value: "{form.read().name}",
                        style: INPUT_STYLE,
                        oninput: move |e| form.write().name = e.value(),
                    }
                }
                div {
                    label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Description" }
                    input {
                        r#type: "text",
                        placeholder: "Optional",
                        value: "{form.read().description}",
                        style: INPUT_STYLE,
                        oninput: move |e| form.write().description = e.value(),
                    }
                }
            }

            // Row 2: Trigger + Interval
            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 12px;",
                div {
                    label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Trigger" }
                    select {
                        style: INPUT_STYLE,
                        onchange: move |e| form.write().trigger_kind = e.value(),
                        option { value: "", "— select —" }
                        option { value: "startup",  "On startup" }
                        option { value: "interval", "Interval" }
                    }
                }
                if show_interval {
                    div {
                        label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Every (seconds)" }
                        input {
                            r#type: "number",
                            placeholder: "300",
                            value: "{form.read().interval_secs}",
                            style: INPUT_STYLE,
                            oninput: move |e| form.write().interval_secs = e.value(),
                        }
                    }
                }
            }

            // Row 3: Action + Service/Command
            div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 16px;",
                div {
                    label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Action" }
                    select {
                        style: INPUT_STYLE,
                        onchange: move |e| form.write().action_kind = e.value(),
                        option { value: "",        "— select —" }
                        option { value: "start",   "Start container" }
                        option { value: "stop",    "Stop container" }
                        option { value: "restart", "Restart container" }
                        option { value: "command", "Run command" }
                    }
                }
                div {
                    label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;",
                        "{svc_label}"
                    }
                    input {
                        r#type: "text",
                        placeholder: "{svc_hint}",
                        value: "{form.read().service_or_cmd}",
                        style: INPUT_STYLE,
                        oninput: move |e| form.write().service_or_cmd = e.value(),
                    }
                }
            }

            button {
                disabled: !form_valid,
                style: "padding: 8px 20px; background: var(--fsn-color-primary); color: white; \
                        border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                onclick: move |_| {
                    let built = form.read().build_bot();
                    if let Some(bot) = built {
                        bots.write().push(bot);
                        save_bots(bots, status_msg);
                        show_add.set(false);
                        form.set(AddBotForm::default());
                    }
                },
                "Add Bot"
            }
        }
    }
}

// ── shared style constants ────────────────────────────────────────────────────

const INPUT_STYLE: &str = "width: 100%; padding: 6px 10px; \
    border: 1px solid var(--fsn-color-border-default); \
    border-radius: var(--fsn-radius-md); font-size: 13px;";
