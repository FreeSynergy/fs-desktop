/// Plugin builder — scaffolds WASM plugins with wit-bindgen interfaces.
use dioxus::prelude::*;

/// Plugin builder component.
#[component]
pub fn PluginBuilder() -> Element {
    let plugin_name = use_signal(String::new);
    let plugin_type = use_signal(|| "bridge".to_string());

    rsx! {
        div {
            class: "fsd-plugin-builder",
            style: "padding: 24px;",

            h3 { "Plugin Builder" }
            p { style: "color: var(--fsn-color-text-muted); margin-bottom: 16px;",
                "Scaffold a new WASM plugin using wit-bindgen interfaces. The generated code compiles to a .wasm file that can be distributed via the FSN store."
            }

            div { style: "margin-bottom: 16px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Plugin Name" }
                input {
                    r#type: "text",
                    placeholder: "e.g. my-bridge",
                    style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);",
                    oninput: move |e| *plugin_name.write() = e.value(),
                }
            }

            div { style: "margin-bottom: 16px;",
                label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Plugin Type" }
                select {
                    style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);",
                    onchange: move |e| *plugin_type.write() = e.value(),
                    option { value: "bridge", "Bridge (connects two services)" }
                    option { value: "health", "Health Check (custom health logic)" }
                    option { value: "theme",  "Theme (visual customization)" }
                    option { value: "i18n",   "Language Pack (translations)" }
                    option { value: "custom", "Custom" }
                }
            }

            div { style: "padding: 16px; background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); border: 1px solid var(--fsn-color-border-default); margin-bottom: 16px;",
                p { style: "margin: 0; font-size: 13px; font-family: var(--fsn-font-mono);",
                    "# Generated: {plugin_name.read()}-plugin.wit"
                }
                p { style: "margin: 4px 0 0; color: var(--fsn-color-text-muted); font-size: 12px;",
                    "wit-bindgen interface will be generated here"
                }
            }

            button {
                style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                disabled: plugin_name.read().is_empty(),
                "Generate Scaffold"
            }
        }
    }
}
