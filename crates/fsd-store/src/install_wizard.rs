/// Install wizard — step-by-step configuration before first start.
use dioxus::prelude::*;

use crate::package_card::PackageEntry;

#[derive(Clone, PartialEq, Debug)]
pub enum WizardStep {
    Overview,
    Configure,
    ServiceRoles,
    Confirm,
    Installing,
    Done,
}

impl WizardStep {
    pub fn label(&self) -> &str {
        match self {
            Self::Overview    => "Overview",
            Self::Configure   => "Configure",
            Self::ServiceRoles=> "Service Roles",
            Self::Confirm     => "Confirm",
            Self::Installing  => "Installing",
            Self::Done        => "Done",
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Overview     => Some(Self::Configure),
            Self::Configure    => Some(Self::ServiceRoles),
            Self::ServiceRoles => Some(Self::Confirm),
            Self::Confirm      => Some(Self::Installing),
            Self::Installing   => Some(Self::Done),
            Self::Done         => None,
        }
    }
}

/// Install wizard — guides the user through pre-install configuration.
#[component]
pub fn InstallWizard(package: PackageEntry, on_cancel: EventHandler<()>) -> Element {
    let step = use_signal(|| WizardStep::Overview);

    let steps = [
        WizardStep::Overview,
        WizardStep::Configure,
        WizardStep::ServiceRoles,
        WizardStep::Confirm,
    ];

    rsx! {
        div {
            class: "fsd-install-wizard",
            style: "display: flex; flex-direction: column; height: 100%;",

            // Step indicator
            div {
                style: "display: flex; align-items: center; padding: 16px; border-bottom: 1px solid var(--fsn-color-border-default);",
                for (i, s) in steps.iter().enumerate() {
                    div {
                        style: "display: flex; align-items: center; gap: 4px;",
                        div {
                            style: "width: 24px; height: 24px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 12px; background: {if *step.read() == *s { \"var(--fsn-color-primary)\" } else { \"var(--fsn-color-bg-overlay)\" }}; color: {if *step.read() == *s { \"white\" } else { \"var(--fsn-color-text-muted)\" }};",
                            "{i + 1}"
                        }
                        span {
                            style: "font-size: 13px; color: {if *step.read() == *s { \"var(--fsn-color-text-primary)\" } else { \"var(--fsn-color-text-muted)\" }};",
                            "{s.label()}"
                        }
                        if i < steps.len() - 1 {
                            span { style: "margin: 0 8px; color: var(--fsn-color-text-muted);", "›" }
                        }
                    }
                }
            }

            // Step content
            div {
                style: "flex: 1; overflow: auto; padding: 24px;",
                match *step.read() {
                    WizardStep::Overview => rsx! {
                        h3 { "Install {package.name}" }
                        p { "{package.description}" }
                        p { style: "color: var(--fsn-color-text-muted); font-size: 13px;", "Version: {package.version}" }
                    },
                    WizardStep::Configure => rsx! {
                        h3 { "Configure {package.name}" }
                        p { style: "color: var(--fsn-color-text-muted);", "Configuration options are loaded from the package manifest." }
                        // TODO: render dynamic form from package schema
                    },
                    WizardStep::ServiceRoles => rsx! {
                        h3 { "Service Roles" }
                        p { style: "color: var(--fsn-color-text-muted);",
                            "This package can fulfill the following service roles. Assign them or skip."
                        }
                        // TODO: show role assignment UI
                    },
                    WizardStep::Confirm => rsx! {
                        h3 { "Ready to install" }
                        p { "Click Install to download and start {package.name}." }
                    },
                    WizardStep::Installing => rsx! {
                        div { style: "text-align: center; padding: 32px;",
                            p { "Installing {package.name}…" }
                            // TODO: progress indicator
                        }
                    },
                    WizardStep::Done => rsx! {
                        div { style: "text-align: center; padding: 32px;",
                            p { style: "font-size: 24px;", "✓" }
                            p { "{package.name} installed successfully." }
                        }
                    },
                }
            }

            // Navigation buttons
            div {
                style: "display: flex; justify-content: space-between; padding: 16px; border-top: 1px solid var(--fsn-color-border-default);",
                button {
                    style: "padding: 8px 16px; background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: pointer;",
                    onclick: move |_| on_cancel.call(()),
                    "Cancel"
                }
                button {
                    style: "padding: 8px 16px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                    onclick: move |_| {
                        if let Some(next) = step.read().next() {
                            *step.write() = next;
                        }
                    },
                    if *step.read() == WizardStep::Confirm { "Install" } else { "Next →" }
                }
            }
        }
    }
}
