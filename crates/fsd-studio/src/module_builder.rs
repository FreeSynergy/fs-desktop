/// Module builder — converts Docker Compose / YAML into an FSN module definition.
use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug, Default)]
pub enum BuilderStep {
    #[default]
    Input,
    Review,
    Export,
}

/// Module builder component.
#[component]
pub fn ModuleBuilder() -> Element {
    let step = use_signal(BuilderStep::default);
    let yaml_input = use_signal(String::new);
    let module_name = use_signal(String::new);

    rsx! {
        div {
            class: "fsd-module-builder",
            style: "padding: 24px;",

            match *step.read() {
                BuilderStep::Input => rsx! {
                    h3 { "Module Builder" }
                    p { style: "color: var(--fsn-color-text-muted); margin-bottom: 16px;",
                        "Paste a Docker Compose file, container YAML, or image name. The builder will generate an FSN module definition."
                    }

                    div { style: "margin-bottom: 16px;",
                        label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "Module Name" }
                        input {
                            r#type: "text",
                            placeholder: "e.g. my-service",
                            style: "width: 100%; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md);",
                            oninput: move |e| *module_name.write() = e.value(),
                        }
                    }

                    div { style: "margin-bottom: 16px;",
                        label { style: "display: block; font-weight: 500; margin-bottom: 4px;", "YAML / Compose Input" }
                        textarea {
                            style: "width: 100%; height: 240px; padding: 8px 12px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-family: var(--fsn-font-mono); font-size: 13px; resize: vertical;",
                            placeholder: "version: '3'\nservices:\n  myapp:\n    image: myapp:latest\n    ports:\n      - '8080:8080'",
                            oninput: move |e| *yaml_input.write() = e.value(),
                            "{yaml_input.read()}"
                        }
                    }

                    button {
                        style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                        disabled: yaml_input.read().is_empty() || module_name.read().is_empty(),
                        onclick: move |_| *step.write() = BuilderStep::Review,
                        "Analyse →"
                    }
                },

                BuilderStep::Review => rsx! {
                    h3 { "Review Generated Module" }
                    p { style: "color: var(--fsn-color-text-muted);",
                        "Review and adjust the generated FSN module definition before exporting."
                    }
                    // TODO: show parsed module fields, allow editing
                    div { style: "display: flex; gap: 8px; margin-top: 16px;",
                        button {
                            style: "padding: 8px 16px; background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: pointer;",
                            onclick: move |_| *step.write() = BuilderStep::Input,
                            "← Back"
                        }
                        button {
                            style: "padding: 8px 24px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                            onclick: move |_| *step.write() = BuilderStep::Export,
                            "Export →"
                        }
                    }
                },

                BuilderStep::Export => rsx! {
                    h3 { "Export Module" }
                    p { style: "color: var(--fsn-color-text-muted);",
                        "Save the module to your local store or submit it to FreeSynergy.Node.Store."
                    }
                    // TODO: export options
                    button {
                        style: "padding: 8px 16px; background: var(--fsn-color-bg-surface); border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); cursor: pointer;",
                        onclick: move |_| *step.write() = BuilderStep::Review,
                        "← Back"
                    }
                },
            }
        }
    }
}
