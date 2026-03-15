/// Account settings — connected OIDC providers for single sign-on.
///
/// Providers are stored in `~/.config/fsn/accounts.toml`. Each entry holds the
/// provider name, discovery URL, and client ID; tokens themselves are never
/// stored here — they are negotiated at runtime via fsn-auth.
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

// ── Data model ────────────────────────────────────────────────────────────────

/// A configured OIDC provider entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OidcProvider {
    /// Short display name (e.g. "Kanidm", "Keycloak").
    pub name: String,
    /// OIDC discovery base URL (e.g. `https://auth.example.com`).
    pub discovery_url: String,
    /// OAuth2 client ID registered with this provider.
    pub client_id: String,
    /// Scopes to request (space-separated, e.g. `"openid email profile"`).
    pub scopes: String,
    /// Whether this provider is currently active.
    pub enabled: bool,
}

#[derive(Default, Serialize, Deserialize)]
struct AccountsConfig {
    #[serde(default)]
    providers: Vec<OidcProvider>,
}

impl AccountsConfig {
    fn path() -> std::path::PathBuf {
        crate::config_path("accounts.toml")
    }

    fn load() -> Vec<OidcProvider> {
        let content = std::fs::read_to_string(Self::path()).unwrap_or_default();
        toml::from_str::<AccountsConfig>(&content)
            .unwrap_or_default()
            .providers
    }

    fn save(providers: &[OidcProvider]) -> Result<(), String> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let cfg = AccountsConfig { providers: providers.to_vec() };
        let content = toml::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}

// ── Add-provider form ─────────────────────────────────────────────────────────

#[derive(Clone, Default)]
struct AddProviderForm {
    name: String,
    discovery_url: String,
    client_id: String,
    scopes: String,
}

impl AddProviderForm {
    fn is_valid(&self) -> bool {
        !self.name.trim().is_empty()
            && !self.discovery_url.trim().is_empty()
            && !self.client_id.trim().is_empty()
    }

    fn build(&self) -> OidcProvider {
        OidcProvider {
            name: self.name.trim().to_string(),
            discovery_url: self.discovery_url.trim().to_string(),
            client_id: self.client_id.trim().to_string(),
            scopes: if self.scopes.trim().is_empty() {
                "openid email profile".to_string()
            } else {
                self.scopes.trim().to_string()
            },
            enabled: true,
        }
    }
}

// ── Component ─────────────────────────────────────────────────────────────────

/// Account settings — manage connected OIDC providers.
#[component]
pub fn AccountSettings() -> Element {
    let mut providers = use_signal(AccountsConfig::load);
    let mut show_add = use_signal(|| false);
    let mut form = use_signal(AddProviderForm::default);
    let mut status_msg: Signal<Option<String>> = use_signal(|| None);

    let mut save = move || {
        match AccountsConfig::save(&providers.read()) {
            Ok(()) => *status_msg.write() = None,
            Err(e) => *status_msg.write() = Some(format!("Save error: {e}")),
        }
    };

    rsx! {
        div {
            class: "fsd-accounts",
            style: "padding: 24px; max-width: 640px;",

            // Header
            div {
                style: "display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 20px;",
                div {
                    h3 { style: "margin: 0 0 4px;", "Accounts" }
                    p { style: "margin: 0; font-size: 13px; color: var(--fsn-color-text-muted);",
                        "Configure OIDC providers for single sign-on across your services."
                    }
                }
                button {
                    style: "padding: 7px 14px; background: var(--fsn-color-primary); color: white; \
                            border: none; border-radius: var(--fsn-radius-md); cursor: pointer; font-size: 13px; white-space: nowrap;",
                    onclick: move |_| {
                        let cur = *show_add.read();
                        *show_add.write() = !cur;
                        *form.write() = AddProviderForm::default();
                    },
                    if *show_add.read() { "Cancel" } else { "+ Connect Provider" }
                }
            }

            // Add-provider form
            if *show_add.read() {
                div {
                    style: "padding: 16px; background: var(--fsn-color-bg-surface); \
                            border-radius: var(--fsn-radius-md); border: 1px solid var(--fsn-color-border-default); \
                            margin-bottom: 20px;",

                    h4 { style: "margin: 0 0 12px;", "New OIDC Provider" }

                    div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 12px; margin-bottom: 12px;",
                        div {
                            label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Name" }
                            input {
                                r#type: "text", placeholder: "e.g. Kanidm",
                                value: "{form.read().name}",
                                style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-size: 13px;",
                                oninput: move |e| form.write().name = e.value(),
                            }
                        }
                        div {
                            label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Client ID" }
                            input {
                                r#type: "text", placeholder: "e.g. fsn-desktop",
                                value: "{form.read().client_id}",
                                style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-size: 13px;",
                                oninput: move |e| form.write().client_id = e.value(),
                            }
                        }
                    }

                    div { style: "margin-bottom: 12px;",
                        label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Discovery URL" }
                        input {
                            r#type: "url", placeholder: "https://auth.example.com",
                            value: "{form.read().discovery_url}",
                            style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-size: 13px;",
                            oninput: move |e| form.write().discovery_url = e.value(),
                        }
                    }

                    div { style: "margin-bottom: 16px;",
                        label { style: "display: block; font-size: 12px; font-weight: 500; margin-bottom: 4px;", "Scopes" }
                        input {
                            r#type: "text", placeholder: "openid email profile",
                            value: "{form.read().scopes}",
                            style: "width: 100%; padding: 6px 10px; border: 1px solid var(--fsn-color-border-default); border-radius: var(--fsn-radius-md); font-size: 13px;",
                            oninput: move |e| form.write().scopes = e.value(),
                        }
                        p { style: "margin: 4px 0 0; font-size: 11px; color: var(--fsn-color-text-muted);",
                            "Defaults to \"openid email profile\" if left blank."
                        }
                    }

                    button {
                        disabled: !form.read().is_valid(),
                        style: "padding: 7px 20px; background: var(--fsn-color-primary); color: white; \
                                border: none; border-radius: var(--fsn-radius-md); cursor: pointer; font-size: 13px;",
                        onclick: move |_| {
                            let provider = form.read().build();
                            providers.write().push(provider);
                            save();
                            *show_add.write() = false;
                            *form.write() = AddProviderForm::default();
                        },
                        "Save Provider"
                    }
                }
            }

            // Provider list
            if providers.read().is_empty() {
                div {
                    style: "text-align: center; padding: 40px; background: var(--fsn-color-bg-surface); \
                            border-radius: var(--fsn-radius-md); border: 1px dashed var(--fsn-color-border-default); \
                            margin-bottom: 16px;",
                    p { style: "color: var(--fsn-color-text-muted); margin: 0;", "No providers connected yet." }
                    p { style: "font-size: 12px; color: var(--fsn-color-text-muted); margin: 8px 0 0;",
                        "Connect an OIDC provider (e.g. Kanidm) to enable SSO for your services."
                    }
                }
            }

            for (idx, provider) in providers.read().iter().enumerate() {
                {
                    let provider = provider.clone();
                    let enabled = provider.enabled;
                    let opacity = if enabled { "1" } else { "0.55" };
                    rsx! {
                        div {
                            key: "{idx}",
                            style: "display: flex; align-items: center; gap: 12px; padding: 12px 14px; \
                                    background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); \
                                    margin-bottom: 8px; border: 1px solid var(--fsn-color-border-default); \
                                    opacity: {opacity};",

                            // Enable toggle
                            input {
                                r#type: "checkbox",
                                checked: enabled,
                                style: "cursor: pointer; width: 16px; height: 16px; flex-shrink: 0;",
                                onchange: move |_| {
                                    let cur = providers.read()[idx].enabled;
                                    providers.write()[idx].enabled = !cur;
                                    save();
                                },
                            }

                            // Icon
                            div {
                                style: "width: 36px; height: 36px; border-radius: var(--fsn-radius-md); \
                                        background: var(--fsn-color-bg-overlay); display: flex; align-items: center; \
                                        justify-content: center; font-size: 18px; flex-shrink: 0;",
                                "🔐"
                            }

                            // Info
                            div { style: "flex: 1; min-width: 0;",
                                div { style: "font-weight: 500; font-size: 14px;", "{provider.name}" }
                                div {
                                    style: "font-size: 12px; color: var(--fsn-color-text-muted); \
                                            overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                                    "{provider.discovery_url}"
                                }
                                div { style: "font-size: 11px; color: var(--fsn-color-text-muted); margin-top: 2px;",
                                    "client_id: {provider.client_id}  ·  scopes: {provider.scopes}"
                                }
                            }

                            // Disconnect
                            button {
                                style: "color: var(--fsn-color-error); background: none; border: none; \
                                        cursor: pointer; font-size: 18px; flex-shrink: 0;",
                                title: "Disconnect",
                                onclick: move |_| {
                                    providers.write().remove(idx);
                                    save();
                                },
                                "✕"
                            }
                        }
                    }
                }
            }

            if let Some(msg) = status_msg.read().as_deref() {
                p { style: "font-size: 12px; color: var(--fsn-color-error); margin-top: 8px;", "{msg}" }
            }
        }
    }
}
