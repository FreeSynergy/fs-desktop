/// Conductor — main app component for container/service/bot management.
use dioxus::prelude::*;

use crate::bot_management::BotManagement;
use crate::dep_graph::DependencyGraph;
use crate::log_viewer::LogViewer;
use crate::resource_view::ResourceView;
use crate::service_list::ServiceList;

/// Active section in the Conductor.
#[derive(Clone, PartialEq, Debug)]
pub enum ConductorSection {
    Services,
    Bots,
    Resources,
    Logs,
    Graph,
}

impl ConductorSection {
    pub fn label(&self) -> &str {
        match self {
            Self::Services  => "Services",
            Self::Bots      => "Bots",
            Self::Resources => "Resources",
            Self::Logs      => "Logs",
            Self::Graph     => "Dependency Graph",
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Services  => "⚙",
            Self::Bots      => "🤖",
            Self::Resources => "📊",
            Self::Logs      => "📋",
            Self::Graph     => "🔗",
        }
    }
}

const ALL_SECTIONS: &[ConductorSection] = &[
    ConductorSection::Services,
    ConductorSection::Bots,
    ConductorSection::Resources,
    ConductorSection::Logs,
    ConductorSection::Graph,
];

/// Root component of the Conductor app.
#[component]
pub fn ConductorApp() -> Element {
    let active = use_signal(|| ConductorSection::Services);
    let selected_service: Signal<Option<String>> = use_signal(|| None);

    rsx! {
        div {
            class: "fsd-conductor",
            style: "display: flex; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fsn-color-bg-base);",

            // ── Left nav sidebar (master) ─────────────────────────────────────
            nav {
                style: "width: 200px; flex-shrink: 0; overflow-y: auto; \
                        background: var(--fsn-color-bg-surface, #0f172a); \
                        border-right: 1px solid var(--fsn-color-border-default, #334155); \
                        padding: 12px 8px;",

                div {
                    style: "margin: 0 0 12px 8px; font-size: 11px; font-weight: 600; \
                            text-transform: uppercase; letter-spacing: 0.08em; \
                            color: var(--fsn-color-text-muted, #64748b);",
                    "Conductor"
                }

                for section in ALL_SECTIONS {
                    ConductorNavBtn {
                        section: (*section).clone(),
                        active,
                    }
                }
            }

            // ── Detail area ───────────────────────────────────────────────────
            div {
                class: "fsd-conductor__detail fsd-page-enter",
                style: "flex: 1; overflow: auto; padding: 16px;",

                match *active.read() {
                    ConductorSection::Services  => rsx! { ServiceList { selected: selected_service } },
                    ConductorSection::Bots      => rsx! { BotManagement {} },
                    ConductorSection::Resources => rsx! { ResourceView {} },
                    ConductorSection::Graph     => rsx! { DependencyGraph {} },
                    ConductorSection::Logs      => rsx! {
                        LogViewer {
                            service: selected_service.read().clone().unwrap_or_default()
                        }
                    },
                }
            }
        }
    }
}

/// A single nav button in the Conductor sidebar.
#[component]
fn ConductorNavBtn(
    section: ConductorSection,
    mut active: Signal<ConductorSection>,
) -> Element {
    let is_active = *active.read() == section;
    let bg     = if is_active { "var(--fsn-color-bg-overlay, #1e293b)" } else { "transparent" };
    let color  = if is_active { "var(--fsn-color-primary, #06b6d4)" } else { "var(--fsn-color-text-primary, #e2e8f0)" };
    let border = if is_active { "2px solid var(--fsn-color-primary, #06b6d4)" } else { "2px solid transparent" };
    rsx! {
        button {
            style: "display: flex; align-items: center; gap: 10px; width: 100%; \
                    padding: 8px 12px; border: none; border-left: {border}; border-radius: 6px; \
                    cursor: pointer; font-size: 14px; text-align: left; \
                    background: {bg}; color: {color}; margin-bottom: 2px;",
            onclick: move |_| *active.write() = section.clone(),
            span { style: "font-size: 16px;", "{section.icon()}" }
            span { "{section.label()}" }
        }
    }
}
