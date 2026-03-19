/// Package detail panel — shown when the user clicks a package card.
///
/// Displays icon, name, description, tags, capability badges, metadata, and an
/// Install/Remove button. Uses InstallPopup for install result feedback.
use dioxus::prelude::*;
use fsd_db::package_registry::PackageRegistry;

use crate::install_wizard::{do_install, InstallPopup, InstallResult};
use crate::missing_icon::MissingIcon;
use crate::node_package::PackageKind;
use crate::package_card::PackageEntry;
use crate::state::notify_install_changed;

// ── PackageDetail ─────────────────────────────────────────────────────────────

/// Full-page or side-panel detail view for a single package.
#[component]
pub fn PackageDetail(
    package: PackageEntry,
    on_back: EventHandler<()>,
    #[props(default)]
    on_select_package: Option<EventHandler<String>>,
) -> Element {
    let mut installing:     Signal<bool>                  = use_signal(|| false);
    let mut install_result: Signal<Option<InstallResult>> = use_signal(|| None);
    let mut installed       = use_signal(|| PackageRegistry::is_installed(&package.id));
    let mut remove_confirm  = use_signal(|| false);

    rsx! {
        div {
            class: "fsd-package-detail fsd-page-fade",
            style: "display: flex; flex-direction: column; height: 100%; \
                    background: var(--fsn-color-bg-base);",

            // Install result popup
            if let Some(result) = install_result.read().clone() {
                {
                    let pkg_id = package.id.clone();
                    rsx! {
                        InstallPopup {
                            result,
                            on_close: move |_| {
                                installed.set(PackageRegistry::is_installed(&pkg_id));
                                install_result.set(None);
                            },
                        }
                    }
                }
            }

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
                                    let pkg_id   = package.id.clone();
                                    let is_bundle = package.kind == PackageKind::Bundle;
                                    move |_| {
                                        if is_bundle {
                                            let _ = PackageRegistry::remove_bundle(&pkg_id);
                                        } else {
                                            let _ = PackageRegistry::remove(&pkg_id);
                                        }
                                        notify_install_changed();
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
                            MissingIcon { size: 48 }
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
                            // Only show Remove if NOT installed via a bundle
                            if package.installed_by.is_none() {
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
                                span {
                                    style: "font-size: 11px; color: var(--fsn-color-text-muted);",
                                    {fsn_i18n::t_with("store.status.installed_via_bundle",
                                        &[("bundle", package.installed_by.as_deref().unwrap_or(""))])}
                                }
                            }
                        } else if *installing.read() {
                            button {
                                style: "padding: 10px 24px; background: var(--fsn-color-bg-overlay); \
                                        border: 1px solid var(--fsn-color-border-default); \
                                        border-radius: var(--fsn-radius-md); cursor: default; \
                                        font-size: 14px;",
                                disabled: true,
                                "⏳ Installing…"
                            }
                        } else {
                            button {
                                style: "padding: 10px 24px; background: var(--fsn-color-primary); \
                                        color: white; border: none; \
                                        border-radius: var(--fsn-radius-md); cursor: pointer; \
                                        font-size: 14px; font-weight: 600;",
                                onclick: {
                                    let pkg = package.clone();
                                    move |_| {
                                        let pkg2 = pkg.clone();
                                        installing.set(true);
                                        spawn(async move {
                                            let result = match do_install(pkg2, String::new()).await {
                                                Ok(()) => {
                                                    notify_install_changed();
                                                    InstallResult::Success
                                                }
                                                Err(e) => InstallResult::Failed(e),
                                            };
                                            installing.set(false);
                                            install_result.set(Some(result));
                                        });
                                    }
                                },
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

                // Bundle: Included Packages section
                if package.kind == PackageKind::Bundle && !package.capabilities.is_empty() {
                    div { style: "margin-bottom: 32px;",
                        h3 {
                            style: "font-size: 14px; font-weight: 600; \
                                    text-transform: uppercase; letter-spacing: 0.06em; \
                                    color: var(--fsn-color-text-muted); margin: 0 0 12px 0;",
                            "Included Packages"
                        }
                        div { style: "display: flex; flex-direction: column; gap: 6px;",
                            for cap in &package.capabilities {
                                {
                                    let cap_id      = cap.clone();
                                    let has_handler = on_select_package.is_some();
                                    let is_member_installed = PackageRegistry::is_installed(&cap_id);
                                    rsx! {
                                        div {
                                            key: "{cap_id}",
                                            style: if has_handler {
                                                "display: flex; align-items: center; gap: 8px; \
                                                 padding: 8px 14px; \
                                                 background: var(--fsn-color-bg-surface); \
                                                 border: 1px solid var(--fsn-color-border-default); \
                                                 border-radius: var(--fsn-radius-md); \
                                                 font-size: 13px; cursor: pointer;"
                                            } else {
                                                "display: flex; align-items: center; gap: 8px; \
                                                 padding: 8px 14px; \
                                                 background: var(--fsn-color-bg-surface); \
                                                 border: 1px solid var(--fsn-color-border-default); \
                                                 border-radius: var(--fsn-radius-md); font-size: 13px;"
                                            },
                                            onclick: move |_| {
                                                if let Some(ref handler) = on_select_package {
                                                    handler.call(cap_id.clone());
                                                }
                                            },
                                            span {
                                                style: "color: var(--fsn-color-text-muted);",
                                                if is_member_installed { "✅" } else { "○" }
                                            }
                                            span { style: "flex: 1;", "{cap}" }
                                            if is_member_installed {
                                                span {
                                                    style: "font-size: 11px; color: var(--fsn-color-success, #22c55e); \
                                                            background: rgba(34,197,94,0.12); \
                                                            padding: 1px 6px; border-radius: 999px;",
                                                    {fsn_i18n::t("store.status.installed")}
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Capabilities
                if package.kind != PackageKind::Bundle && !package.capabilities.is_empty() {
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

                // Completeness checklist
                div { style: "margin-bottom: 32px;",
                    h3 {
                        style: "font-size: 14px; font-weight: 600; \
                                text-transform: uppercase; letter-spacing: 0.06em; \
                                color: var(--fsn-color-text-muted); margin: 0 0 12px 0;",
                        "Completeness"
                    }
                    div {
                        style: "background: var(--fsn-color-bg-surface); \
                                border: 1px solid var(--fsn-color-border-default); \
                                border-radius: var(--fsn-radius-md); overflow: hidden;",
                        CompletenessRow {
                            label: "Icon".to_string(),
                            ok: package.icon.is_some(),
                        }
                        CompletenessRow {
                            label: "Description".to_string(),
                            ok: !package.description.is_empty(),
                        }
                        CompletenessRow {
                            label: "Tags".to_string(),
                            ok: !package.tags.is_empty(),
                        }
                        if package.kind == PackageKind::Language {
                            CompletenessRow {
                                label: "License".to_string(),
                                ok: false,
                                // Language packs don't require a license field — show N/A
                                na: true,
                            }
                        } else {
                            CompletenessRow {
                                label: "License".to_string(),
                                ok: !package.license.is_empty(),
                                na: false,
                            }
                        }
                        CompletenessRow {
                            label: "Author".to_string(),
                            ok: !package.author.is_empty(),
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

// ── CompletenessRow ───────────────────────────────────────────────────────────

#[component]
fn CompletenessRow(
    label: String,
    ok:    bool,
    #[props(default = false)]
    na:    bool,
) -> Element {
    let (icon, color) = if na {
        ("N/A", "var(--fsn-color-text-muted)")
    } else if ok {
        ("✅", "var(--fsn-color-success, #22c55e)")
    } else {
        ("❌", "var(--fsn-color-error, #ef4444)")
    };

    rsx! {
        div {
            style: "display: flex; padding: 9px 16px; \
                    border-bottom: 1px solid var(--fsn-color-border-default); \
                    font-size: 13px; align-items: center; gap: 10px;",
            span {
                style: "min-width: 100px; color: var(--fsn-color-text-muted);",
                "{label}"
            }
            span { style: "color: {color};", "{icon}" }
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
