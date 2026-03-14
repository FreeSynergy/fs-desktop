/// Account settings — connected OIDC accounts.
use dioxus::prelude::*;

/// An OIDC account connection.
#[derive(Clone, Debug, PartialEq)]
pub struct ConnectedAccount {
    pub provider: String,
    pub username: String,
    pub email: String,
    pub scopes: Vec<String>,
}

/// Account settings component.
#[component]
pub fn AccountSettings() -> Element {
    let accounts = use_signal(Vec::<ConnectedAccount>::new);

    rsx! {
        div {
            class: "fsd-accounts",
            style: "padding: 24px; max-width: 600px;",

            h3 { style: "margin-top: 0;", "Accounts" }
            p { style: "color: var(--fsn-color-text-muted); margin-bottom: 24px;",
                "Connect external OIDC providers for single sign-on across your services."
            }

            if accounts.read().is_empty() {
                div {
                    style: "text-align: center; padding: 32px; background: var(--fsn-color-bg-surface); border-radius: var(--fsn-radius-md); border: 1px dashed var(--fsn-color-border-default); margin-bottom: 16px;",
                    p { style: "color: var(--fsn-color-text-muted);", "No accounts connected yet." }
                }
            }

            button {
                style: "padding: 8px 16px; background: var(--fsn-color-primary); color: white; border: none; border-radius: var(--fsn-radius-md); cursor: pointer;",
                "+ Connect Account"
            }
        }
    }
}
