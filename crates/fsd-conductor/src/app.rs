/// Conductor — main app component for container/service/bot management.
use dioxus::prelude::*;
use fsn_components::{FsnSidebar, FsnSidebarItem, FSN_SIDEBAR_CSS};

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

    /// Look up a section by its label string.
    pub fn from_label(label: &str) -> Option<Self> {
        match label {
            "Services"          => Some(Self::Services),
            "Bots"              => Some(Self::Bots),
            "Resources"         => Some(Self::Resources),
            "Logs"              => Some(Self::Logs),
            "Dependency Graph"  => Some(Self::Graph),
            _                   => None,
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

    let sidebar_items: Vec<FsnSidebarItem> = ALL_SECTIONS.iter()
        .map(|s| FsnSidebarItem::new(s.label(), s.icon(), s.label()))
        .collect();

    rsx! {
        style { "{FSN_SIDEBAR_CSS}" }
        div {
            class: "fsd-conductor",
            style: "display: flex; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fsn-color-bg-base);",

            // ── Left nav sidebar (collapsible) ────────────────────────────────
            FsnSidebar {
                items:     sidebar_items,
                active_id: active.read().label().to_string(),
                on_select: move |id: String| {
                    if let Some(section) = ConductorSection::from_label(&id) {
                        active.set(section);
                    }
                },
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
