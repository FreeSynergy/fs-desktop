/// Studio — root component: module builder, plugin builder, i18n editor.
use dioxus::prelude::*;

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
    let active_tab = use_signal(|| StudioTab::Modules);

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
                StudioTabBtn { label: "Module Builder", tab: StudioTab::Modules,  active: active_tab }
                StudioTabBtn { label: "Plugin Builder", tab: StudioTab::Plugins,  active: active_tab }
                StudioTabBtn { label: "i18n Editor",    tab: StudioTab::I18n,     active: active_tab }
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

#[component]
fn StudioTabBtn(label: &'static str, tab: StudioTab, mut active: Signal<StudioTab>) -> Element {
    let is_active = *active.read() == tab;
    rsx! {
        button {
            style: "padding: 10px 20px; border: none; cursor: pointer; font-size: 14px; background: {if is_active { \"var(--fsn-color-bg-base)\" } else { \"transparent\" }}; border-bottom: 2px solid {if is_active { \"var(--fsn-color-primary)\" } else { \"transparent\" }};",
            onclick: move |_| *active.write() = tab.clone(),
            "{label}"
        }
    }
}
