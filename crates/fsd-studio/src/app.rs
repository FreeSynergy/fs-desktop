/// Studio — root component: module builder, plugin builder, i18n editor.
use dioxus::prelude::*;
use fsn_components::TabBtn;

use crate::module_builder::ModuleBuilder;
use crate::plugin_builder::PluginBuilder;
use crate::i18n_editor::I18nEditor;

#[derive(Clone, PartialEq, Debug)]
pub enum StudioTab {
    Modules,
    Plugins,
    I18n,
}

/// Root Studio component.
#[component]
pub fn StudioApp() -> Element {
    let mut active_tab = use_signal(|| StudioTab::Modules);

    rsx! {
        div {
            class: "fsd-studio",
            style: "display: flex; flex-direction: column; height: 100%; background: var(--fsn-color-bg-base);",

            // Header
            div {
                style: "padding: 16px; background: var(--fsn-color-bg-surface); border-bottom: 1px solid var(--fsn-color-border-default);",
                h2 { style: "margin: 0; font-size: 20px;", "Studio" }
                p { style: "margin: 4px 0 0; color: var(--fsn-color-text-muted); font-size: 13px;",
                    "Create modules, plugins, and language files for the FreeSynergy ecosystem."
                }
            }

            // Tab bar
            div {
                style: "display: flex; border-bottom: 1px solid var(--fsn-color-border-default);",
                TabBtn { label: "Module Builder", is_active: *active_tab.read() == StudioTab::Modules, on_click: move |_| active_tab.set(StudioTab::Modules) }
                TabBtn { label: "Plugin Builder", is_active: *active_tab.read() == StudioTab::Plugins, on_click: move |_| active_tab.set(StudioTab::Plugins) }
                TabBtn { label: "i18n Editor",    is_active: *active_tab.read() == StudioTab::I18n,    on_click: move |_| active_tab.set(StudioTab::I18n) }
            }

            // Content
            div {
                style: "flex: 1; overflow: auto;",
                match *active_tab.read() {
                    StudioTab::Modules => rsx! { ModuleBuilder {} },
                    StudioTab::Plugins => rsx! { PluginBuilder {} },
                    StudioTab::I18n    => rsx! { I18nEditor {} },
                }
            }
        }
    }
}

