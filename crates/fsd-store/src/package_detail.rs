/// Package detail panel — shown when the user clicks a package card.
///
/// Displays icon, name, description, tags, capability badges, metadata, and an
/// Install/Remove button. Opens the InstallWizard for uninstalled packages.
use dioxus::prelude::*;
use fsd_db::package_registry::PackageRegistry;

use crate::install_wizard::InstallWizard;
use crate::package_card::PackageEntry;

// ── PackageDetail ─────────────────────────────────────────────────────────────

/// Full-page or side-panel detail view for a single package.
#[component]
pub fn PackageDetail(package: PackageEntry, on_back: EventHandler<()>) -> Element {
    let mut show_wizard   = use_signal(|| false);
    let mut installed     = use_signal(|| package.installed);
    let mut remove_confirm = use_signal(|| false);

    if *show_wizard.read() {
        return rsx! {
            InstallWizard {
                package: package.clone(),
                on_cancel: move |_| {
                    show_wizard.set(false);
                    installed.set(PackageRegistry::is_installed(&package.id));
                },
            }
        };
    }

    rsx! {
        div {
            class: "fsd-package-detail fsd-page-fade",
            style: "display: flex; flex-direction: column; height: 100%; \
                    background: var(--fsn-color-bg-base);",

            // Remove confirm dialog
            if *remove_confirm.read() {
                div {
                    style: "position: fixed; inset: 0; background: rgba(0,0,0,0.5); \
                            display: flex; align-items: center; justify-content: center; z-index: 1000;",
                    div {
                        style: "background: var(--fsn-color-bg-surface); \
                                border: 1px solid var(--fsn-color-border-default); \
                                border-radius: var(--fsn-radius-lg); padding: 24px; \
                                max-width: 400px; width: 100%;",
                        h3 { style: "margin: 0 0 12px 0;",
                            {fsn_i18n::t_with("store.dialog.remove_title", &[("name", package.name.as_str())])}
                        }
                        p {
                            style: "color: var(--fsn-color-text-muted); font-size: 14px; margin-bottom: 20px;",
                            {fsn_i18n::t("store.dialog.remove_body")}
                        }
                        div {
                            style: "display: flex; gap: 8px; justify-content: flex-end;",
                            button {
                                style: "padding: 8px 16px; background: var(--fsn-color-bg-surface); \
                                        border: 1px solid var(--fsn-color-border-default); \
                                        border-radius: var(--fsn-radius-md); cursor: pointer;",
                                onclick: move |_| remove_confirm.set(false),
                                {fsn_i18n::t("actions.cancel")}
                            }
                            button {
                                style: "padding: 8px 16px; background: var(--fsn-color-error, #ef4444); \
                                        color: white; border: none; \
                                        border-radius: var(--fsn-radius-md); cursor: pointer;",
                                onclick: {
                                    let pkg_id = package.id.clone();
                                    move |_| {
                                        let _ = PackageRegistry::remove(&pkg_id);
                                        installed.set(false);
                                        remove_confirm.set(false);
                                    }
                                },
                                {fsn_i18n::t("actions.remove")}
                            }
                        }
                    }
                }
            }

            // ── Top bar ──────────────────────────────────────────────────────
            div {
                style: "display: flex; align-items: center; gap: 12px; padding: 14px 20px; \
                        border-bottom: 1px solid var(--fsn-color-border-default); \
                        background: var(--fsn-color-bg-surface);",
                button {
                    style: "background: none; border: none; cursor: pointer; \
                            color: var(--fsn-color-text-muted); font-size: 20px; padding: 0 4px;",
                    title: fsn_i18n::t("store.back_to_browser"),
                    onclick: move |_| on_back.call(()),
                    "‹"
                }
                span {
                    style: "font-size: 14px; color: var(--fsn-color-text-muted);",
                    {fsn_i18n::t("store.title")}
                }
                span { style: "color: var(--fsn-color-text-muted);", "/" }
                span {
                    style: "font-size: 14px; color: var(--fsn-color-text-primary);",
                    "{package.name}"
                }
            }

            // ── Scrollable body ──────────────────────────────────────────────
            div {
                style: "flex: 1; overflow-y: auto; padding: 32px;",

                // Header row: icon + title + meta
                div {
                    style: "display: flex; align-items: flex-start; gap: 24px; margin-bottom: 32px;",

                    // Icon
                    div {
                        style: "width: 72px; height: 72px; flex-shrink: 0; \
                                border-radius: var(--fsn-radius-lg); \
                                background: var(--fsn-color-bg-surface); \
                                border: 1px solid var(--fsn-color-border-default); \
                                display: flex; align-items: center; justify-content: center; \
                                font-size: 36px;",
                        if let Some(icon_url) = &package.icon {
                            img {
                                src: "{icon_url}",
                                width: "48",
                                height: "48",
                                style: "object-fit: contain;",
                            }
                        } else {
                            span { "📦" }
                        }
                    }

                    // Title + meta
                    div { style: "flex: 1;",
                        h2 { style: "margin: 0 0 4px 0; font-size: 24px;", "{package.name}" }
                        p {
                            style: "margin: 0 0 12px 0; font-size: 14px; \
                                    color: var(--fsn-color-text-muted);",
                            "v{package.version} · {package.category}"
                        }
                        // Kind badge + tags
                        div {
                            style: "display: flex; flex-wrap: wrap; gap: 6px;",
                            span {
                                style: "padding: 2px 10px; border-radius: 999px; font-size: 12px; \
                                        background: var(--fsn-color-primary); color: white;",
                                "{package.kind.label()}"
                            }
                            for tag in &package.tags {
                                span {
                                    key: "{tag}",
                                    style: "padding: 2px 10px; border-radius: 999px; font-size: 12px; \
                                            background: var(--fsn-color-bg-surface); \
                                            border: 1px solid var(--fsn-color-border-default); \
                                            color: var(--fsn-color-text-muted);",
                                    "{tag}"
                                }
                            }
                        }
                    }

                    // Install / Remove button (top-right)
                    div { style: "display: flex; flex-direction: column; gap: 8px; align-items: flex-end;",
                        if *installed.read() {
                            button {
                                style: "padding: 10px 24px; background: var(--fsn-color-bg-overlay); \
                                        border: 1px solid var(--fsn-color-border-default); \
                                        border-radius: var(--fsn-radius-md); cursor: default; \
                                        font-size: 14px;",
                                disabled: true,
                                {fsn_i18n::t("store.status.installed")}
                            }
                            button {
                                style: "padding: 6px 16px; background: transparent; \
                                        border: 1px solid var(--fsn-color-error, #ef4444); \
                                        color: var(--fsn-color-error, #ef4444); \
                                        border-radius: var(--fsn-radius-md); cursor: pointer; \
                                        font-size: 12px;",
                                onclick: move |_| remove_confirm.set(true),
                                {fsn_i18n::t("actions.remove")}
                            }
                        } else {
                            button {
                                style: "padding: 10px 24px; background: var(--fsn-color-primary); \
                                        color: white; border: none; \
                                        border-radius: var(--fsn-radius-md); cursor: pointer; \
                                        font-size: 14px; font-weight: 600;",
                                onclick: move |_| show_wizard.set(true),
                                {fsn_i18n::t("actions.install")}
                            }
                        }
                    }
                }

                // Description
                div { style: "margin-bottom: 32px;",
                    h3 {
                        style: "font-size: 14px; font-weight: 600; \
                                text-transform: uppercase; letter-spacing: 0.06em; \
                                color: var(--fsn-color-text-muted); margin: 0 0 12px 0;",
                        {fsn_i18n::t("labels.description")}
                    }
                    p {
                        style: "font-size: 14px; line-height: 1.7; \
                                color: var(--fsn-color-text-secondary);",
                        "{package.description}"
                    }
                }

                // Capabilities
                if !package.capabilities.is_empty() {
                    div { style: "margin-bottom: 32px;",
                        h3 {
                            style: "font-size: 14px; font-weight: 600; \
                                    text-transform: uppercase; letter-spacing: 0.06em; \
                                    color: var(--fsn-color-text-muted); margin: 0 0 12px 0;",
                            {fsn_i18n::t("store.section.capabilities")}
                        }
                        div {
                            style: "display: flex; flex-wrap: wrap; gap: 8px;",
                            for cap in &package.capabilities {
                                CapabilityBadge { label: cap.clone() }
                            }
                        }
                    }
                }

                // Metadata table
                div {
                    h3 {
                        style: "font-size: 14px; font-weight: 600; \
                                text-transform: uppercase; letter-spacing: 0.06em; \
                                color: var(--fsn-color-text-muted); margin: 0 0 12px 0;",
                        {fsn_i18n::t("store.section.package_info")}
                    }
                    div {
                        style: "background: var(--fsn-color-bg-surface); \
                                border: 1px solid var(--fsn-color-border-default); \
                                border-radius: var(--fsn-radius-md); overflow: hidden;",
                        MetaRow { label: fsn_i18n::t("labels.id"),      value: package.id.clone() }
                        MetaRow { label: fsn_i18n::t("labels.version"), value: package.version.clone() }
                        MetaRow { label: "Category".to_string(),        value: package.category.clone() }
                    }
                }
            }
        }
    }
}

// ── CapabilityBadge ───────────────────────────────────────────────────────────

#[component]
fn CapabilityBadge(label: String) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 6px; \
                    padding: 6px 14px; \
                    background: var(--fsn-color-bg-surface); \
                    border: 1px solid var(--fsn-color-border-default); \
                    border-radius: var(--fsn-radius-md); font-size: 13px;",
            span { "✦" }
            span { "{label}" }
        }
    }
}

// ── MetaRow ───────────────────────────────────────────────────────────────────

#[component]
fn MetaRow(label: String, value: String) -> Element {
    rsx! {
        div {
            style: "display: flex; padding: 10px 16px; \
                    border-bottom: 1px solid var(--fsn-color-border-default); \
                    font-size: 13px;",
            span {
                style: "min-width: 100px; color: var(--fsn-color-text-muted);",
                "{label}"
            }
            span {
                style: "color: var(--fsn-color-text-primary);",
                "{value}"
            }
        }
    }
}
