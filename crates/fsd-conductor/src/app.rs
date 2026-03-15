/// Conductor — main app component for container/service/bot management.
use dioxus::prelude::*;
use fsn_components::SidebarNavBtn;

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
    let mut active = use_signal(|| ConductorSection::Services);
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
                    SidebarNavBtn {
                        key: "{section.label()}",
                        label: section.label().to_string(),
                        icon:  section.icon().to_string(),
                        is_active: *active.read() == *section,
                        left_border: true,
                        on_click: {
                            let s = (*section).clone();
                            move |_| active.set(s.clone())
                        }
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

