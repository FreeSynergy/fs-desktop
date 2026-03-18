/// Container App Manager — manage running/stopped containers, browse store, build packages.
use dioxus::prelude::*;
use fsn_components::{FsnSidebar, FsnSidebarItem, FSN_SIDEBAR_CSS};
use fsn_i18n;

use crate::build_view::BuildView;
use crate::log_viewer::LogViewer;
use crate::service_detail::ServiceDetail;
use crate::service_list::ServiceList;

/// Active section in the Container App Manager.
#[derive(Clone, PartialEq, Debug)]
pub enum ContainerSection {
    Installed,
    InstallNew,
    Build,
    Logs,
}

impl ContainerSection {
    pub fn id(&self) -> &str {
        match self {
            Self::Installed   => "installed",
            Self::InstallNew  => "install_new",
            Self::Build       => "build",
            Self::Logs        => "logs",
        }
    }

    pub fn label(&self) -> String {
        match self {
            Self::Installed  => fsn_i18n::t("container.section.installed"),
            Self::InstallNew => fsn_i18n::t("container.section.install_new"),
            Self::Build      => fsn_i18n::t("container.section.build"),
            Self::Logs       => fsn_i18n::t("container.section.logs"),
        }
    }

    pub fn icon(&self) -> &str {
        match self {
            Self::Installed  => "📦",
            Self::InstallNew => "🛍",
            Self::Build      => "🔧",
            Self::Logs       => "📋",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "installed"   => Some(Self::Installed),
            "install_new" => Some(Self::InstallNew),
            "build"       => Some(Self::Build),
            "logs"        => Some(Self::Logs),
            _             => None,
        }
    }
}

const ALL_SECTIONS: &[ContainerSection] = &[
    ContainerSection::Installed,
    ContainerSection::InstallNew,
    ContainerSection::Build,
    ContainerSection::Logs,
];

/// Root component of the Container App Manager.
#[component]
pub fn ConductorApp() -> Element {
    let mut active = use_signal(|| ContainerSection::Installed);
    let mut selected_service: Signal<Option<String>> = use_signal(|| None);

    let sidebar_items: Vec<FsnSidebarItem> = ALL_SECTIONS.iter()
        .map(|s| FsnSidebarItem::new(s.id(), s.icon(), s.label()))
        .collect();

    rsx! {
        style { "{FSN_SIDEBAR_CSS}" }
        div {
            class: "fsd-conductor",
            style: "display: flex; flex-direction: column; height: 100%; width: 100%; overflow: hidden; \
                    background: var(--fsn-color-bg-base);",

            // App title bar
            div {
                style: "padding: 10px 16px; border-bottom: 1px solid var(--fsn-border); \
                        flex-shrink: 0; background: var(--fsn-bg-surface);",
                h2 {
                    style: "margin: 0; font-size: 16px; font-weight: 600; color: var(--fsn-text-primary);",
                    {fsn_i18n::t("container.title")}
                }
            }

            // Sidebar + Content row
            div {
                style: "display: flex; flex: 1; overflow: hidden;",

                // ── Left nav sidebar ───────────────────────────────────────────────
                FsnSidebar {
                    items:     sidebar_items,
                    active_id: active.read().id().to_string(),
                    on_select: move |id: String| {
                        if let Some(section) = ContainerSection::from_id(&id) {
                            active.set(section);
                        }
                    },
                }

                // ── Detail area ───────────────────────────────────────────────────
                div {
                    class: "fsd-conductor__detail fsd-page-enter",
                    style: "flex: 1; display: flex; overflow: hidden;",

                    match *active.read() {
                        // Installed: split-view — list on left, detail on right
                        ContainerSection::Installed => rsx! {
                            div {
                                style: {
                                    if selected_service.read().is_some() {
                                        "flex: 0 0 340px; overflow: auto; padding: 16px; \
                                         border-right: 1px solid var(--fsn-border);"
                                    } else {
                                        "flex: 1; overflow: auto; padding: 16px;"
                                    }
                                },
                                ServiceList { selected: selected_service }
                            }
                            if let Some(name) = selected_service.read().clone() {
                                div { style: "flex: 1; overflow: hidden;",
                                    ServiceDetail {
                                        service_name: name,
                                        on_close: move |_| *selected_service.write() = None,
                                    }
                                }
                            }
                        },
                        ContainerSection::InstallNew => rsx! {
                            div {
                                style: "flex: 1; overflow: auto; padding: 40px; \
                                        display: flex; align-items: flex-start; justify-content: center;",
                                div {
                                    style: "max-width: 600px; width: 100%; text-align: center; \
                                            color: var(--fsn-color-text-muted);",
                                    p { style: "font-size: 32px; margin-bottom: 16px;", "🛍" }
                                    h3 { style: "margin: 0 0 8px; color: var(--fsn-color-text-primary);",
                                        "Browse Store"
                                    }
                                    p { style: "font-size: 13px;",
                                        "Discover and install new container apps from the FreeSynergy Store."
                                    }
                                    button {
                                        style: "margin-top: 16px; background: var(--fsn-color-primary, #06b6d4); \
                                                color: #fff; border: none; border-radius: var(--fsn-radius-md); \
                                                padding: 10px 24px; font-size: 14px; font-weight: 600; \
                                                cursor: pointer;",
                                        onclick: move |_| {
                                            if let Some(mut req) = try_use_context::<Signal<Option<String>>>() {
                                                *req.write() = Some("store".to_string());
                                            }
                                        },
                                        "Open Store"
                                    }
                                }
                            }
                        },
                        ContainerSection::Build => rsx! {
                            div { style: "flex: 1; overflow: auto; padding: 16px;",
                                BuildView {}
                            }
                        },
                        ContainerSection::Logs => rsx! {
                            div { style: "flex: 1; overflow: auto; padding: 16px;",
                                LogViewer {
                                    service: selected_service.read().clone().unwrap_or_default()
                                }
                            }
                        },
                    }
                }
            } // end sidebar + content row
        }
    }
}
