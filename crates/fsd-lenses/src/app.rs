// Lenses app — L2: Lens-View (grouped by role, Summary + Link → opens in Browser).

use dioxus::prelude::*;
use fsn_i18n;

use crate::model::{Lens, LensRole};
use crate::query::refresh_lens;

// ── Context: Lenses can request opening URLs in the browser ───────────────────

/// Set to `Some(url)` from Lenses; Desktop's Browser picks it up.
pub type BrowserUrlRequest = Signal<Option<String>>;

// ── LensesApp ─────────────────────────────────────────────────────────────────

#[component]
pub fn LensesApp() -> Element {
    let mut lenses: Signal<Vec<Lens>> = use_signal(Vec::new);
    let mut selected: Signal<Option<i64>> = use_signal(|| None);
    let mut show_form = use_signal(|| false);

    // New lens form state
    let mut form_name:  Signal<String> = use_signal(String::new);
    let mut form_query: Signal<String> = use_signal(String::new);

    // Browser URL request context (may not be present if used standalone)
    let browser_req: Option<BrowserUrlRequest> = use_context();

    let open_url = move |url: String| {
        if let Some(mut req) = browser_req {
            req.set(Some(url));
        }
    };

    rsx! {
        style { "{LENSES_CSS}" }

        div {
            class: "fsd-lenses",

            // ── Header ───────────────────────────────────────────────────
            div {
                class: "fsd-lenses__header",
                h2 {
                    style: "margin: 0; font-size: 15px; font-weight: 600; color: var(--fsn-color-text-primary);",
                    {fsn_i18n::t("lenses.title")}
                }
                button {
                    class: "fsd-lenses__btn-primary",
                    onclick: move |_| {
                        form_name.set(String::new());
                        form_query.set(String::new());
                        show_form.set(true);
                    },
                    {fsn_i18n::t("lenses.new_lens")}
                }
            }

            // ── New Lens form ────────────────────────────────────────────
            if *show_form.read() {
                div {
                    class: "fsd-lenses__form",
                    input {
                        class: "fsd-lenses__input",
                        r#type: "text",
                        placeholder: {fsn_i18n::t("lenses.form.name")},
                        value: "{form_name}",
                        oninput: move |e| form_name.set(e.value()),
                    }
                    input {
                        class: "fsd-lenses__input",
                        r#type: "text",
                        placeholder: {fsn_i18n::t("lenses.search_hint")},
                        value: "{form_query}",
                        oninput: move |e| form_query.set(e.value()),
                    }
                    div { style: "display: flex; gap: 8px;",
                        button {
                            class: "fsd-lenses__btn-primary",
                            onclick: move |_| {
                                let name  = form_name.read().trim().to_string();
                                let query = form_query.read().trim().to_string();
                                if !name.is_empty() && !query.is_empty() {
                                    lenses.write().push(Lens::new(name, query));
                                    show_form.set(false);
                                }
                            },
                            {fsn_i18n::t("lenses.form.create")}
                        }
                        button {
                            class: "fsd-lenses__btn-ghost",
                            onclick: move |_| show_form.set(false),
                            {fsn_i18n::t("lenses.form.cancel")}
                        }
                    }
                }
            }

            // ── Main area: list + detail ─────────────────────────────────
            div {
                class: "fsd-lenses__main",

                // Left: lens list
                div {
                    class: "fsd-lenses__list",

                    if lenses.read().is_empty() {
                        p {
                            style: "color: var(--fsn-color-text-muted); font-size: 13px; padding: 16px;",
                            {fsn_i18n::t("lenses.empty")}
                        }
                    }

                    for lens in lenses.read().clone().iter() {
                        LensListRow {
                            key: "{lens.id}",
                            lens: lens.clone(),
                            active: *selected.read() == Some(lens.id),
                            on_select: {
                                let lens_id = lens.id;
                                move |_| selected.set(Some(lens_id))
                            },
                            on_delete: {
                                let lens_id = lens.id;
                                move |_| {
                                    lenses.write().retain(|l| l.id != lens_id);
                                    if *selected.read() == Some(lens_id) {
                                        selected.set(None);
                                    }
                                }
                            },
                            on_refresh: {
                                let lens_clone = lens.clone();
                                move |_| {
                                    let mut lenses = lenses.clone();
                                    let id = lens_clone.id;
                                    // Mark as loading
                                    if let Some(l) = lenses.write().iter_mut().find(|l| l.id == id) {
                                        l.loading = true;
                                    }
                                    let lens_for_task = lens_clone.clone();
                                    spawn(async move {
                                        let items = refresh_lens(&lens_for_task).await;
                                        if let Some(l) = lenses.write().iter_mut().find(|l| l.id == id) {
                                            l.items           = items;
                                            l.loading         = false;
                                            l.last_refreshed  = Some(chrono::Utc::now().to_rfc3339());
                                        }
                                    });
                                }
                            },
                        }
                    }
                }

                // Right: detail view
                div {
                    class: "fsd-lenses__detail",

                    match *selected.read() {
                        None => rsx! {
                            div {
                                style: "display: flex; align-items: center; justify-content: center; height: 100%; \
                                        color: var(--fsn-color-text-muted); font-size: 14px;",
                                "← Select a Lens to view its data"
                            }
                        },
                        Some(id) => {
                            if let Some(lens) = lenses.read().iter().find(|l| l.id == id).cloned() {
                                rsx! {
                                    LensDetail {
                                        lens,
                                        on_open_url: move |url: String| open_url(url),
                                    }
                                }
                            } else {
                                rsx! {}
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── LensListRow ───────────────────────────────────────────────────────────────

#[component]
fn LensListRow(
    lens:       Lens,
    active:     bool,
    on_select:  EventHandler<()>,
    on_delete:  EventHandler<()>,
    on_refresh: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: if active { "fsd-lenses__list-row fsd-lenses__list-row--active" } else { "fsd-lenses__list-row" },
            onclick: move |_| on_select.call(()),

            div { class: "fsd-lenses__list-row-info",
                span {
                    class: "fsd-lenses__list-row-name",
                    "🔍 {lens.name}"
                }
                span {
                    class: "fsd-lenses__list-row-query",
                    "\"{lens.query}\""
                }
                if lens.loading {
                    span {
                        style: "font-size: 11px; color: var(--fsn-color-primary, #06b6d4);",
                        {fsn_i18n::t("lenses.item.loading")}
                    }
                }
            }

            div { class: "fsd-lenses__list-row-actions",
                onclick: |e: MouseEvent| e.stop_propagation(),
                button {
                    class: "fsd-lenses__icon-btn",
                    title: "Refresh",
                    onclick: move |e: MouseEvent| { e.stop_propagation(); on_refresh.call(()); },
                    "↺"
                }
                button {
                    class: "fsd-lenses__icon-btn fsd-lenses__icon-btn--danger",
                    title: {fsn_i18n::t("lenses.delete_lens")},
                    onclick: move |e: MouseEvent| { e.stop_propagation(); on_delete.call(()); },
                    "✕"
                }
            }
        }
    }
}

// ── LensDetail ────────────────────────────────────────────────────────────────

#[component]
fn LensDetail(lens: Lens, on_open_url: EventHandler<String>) -> Element {
    let grouped = lens.grouped();

    rsx! {
        div { class: "fsd-lenses__detail-inner",

            // Title
            h3 {
                style: "margin: 0 0 4px; font-size: 16px; font-weight: 600; color: var(--fsn-color-text-primary);",
                "🔍 {lens.name}"
            }
            p {
                style: "margin: 0 0 16px; font-size: 12px; color: var(--fsn-color-text-muted);",
                "Query: \"{lens.query}\""
                if let Some(ts) = &lens.last_refreshed {
                    span { " · Last refreshed: {ts}" }
                }
            }

            if lens.items.is_empty() {
                if lens.loading {
                    p { style: "color: var(--fsn-color-text-muted);", {fsn_i18n::t("lenses.item.loading")} }
                } else {
                    p { style: "color: var(--fsn-color-text-muted);", {fsn_i18n::t("lenses.item.no_data")} }
                }
            }

            // Groups by role
            for (role, items) in grouped.iter() {
                div { class: "fsd-lenses__group",
                    div { class: "fsd-lenses__group-header",
                        span { style: "font-size: 16px;", "{role.icon()}" }
                        span { class: "fsd-lenses__group-title", "{role.label()}" }
                        span {
                            style: "font-size: 11px; color: var(--fsn-color-text-muted); margin-left: auto;",
                            "{items.len()} result(s)"
                        }
                    }

                    for item in items.iter() {
                        div { class: "fsd-lenses__item",
                            p { class: "fsd-lenses__item-summary", "{item.summary}" }
                            p {
                                style: "margin: 2px 0 0; font-size: 11px; color: var(--fsn-color-text-muted);",
                                "via {item.source}"
                            }
                            if let Some(link) = &item.link {
                                div { style: "margin-top: 6px;",
                                    button {
                                        class: "fsd-lenses__btn-link",
                                        onclick: {
                                            let url = link.clone();
                                            move |_| on_open_url.call(url.clone())
                                        },
                                        "🔗 {fsn_i18n::t(\"lenses.item.open_link\")}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── CSS ───────────────────────────────────────────────────────────────────────

const LENSES_CSS: &str = r#"
.fsd-lenses {
    display: flex;
    flex-direction: column;
    height: 100%;
    width: 100%;
    overflow: hidden;
    background: var(--fsn-color-bg-base);
    font-family: var(--fsn-font-family, system-ui, sans-serif);
}

.fsd-lenses__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 16px;
    border-bottom: 1px solid var(--fsn-color-border-default);
    flex-shrink: 0;
    background: var(--fsn-color-bg-surface);
}

.fsd-lenses__form {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--fsn-color-border-default);
    background: var(--fsn-color-bg-surface);
    flex-shrink: 0;
}

.fsd-lenses__input {
    background: var(--fsn-color-bg-base);
    border: 1px solid var(--fsn-color-border-default);
    border-radius: 6px;
    color: var(--fsn-color-text-primary);
    font-size: 13px;
    padding: 6px 10px;
    outline: none;
}
.fsd-lenses__input:focus {
    border-color: var(--fsn-color-primary, #06b6d4);
}

.fsd-lenses__main {
    display: flex;
    flex: 1;
    overflow: hidden;
}

.fsd-lenses__list {
    width: 260px;
    flex-shrink: 0;
    border-right: 1px solid var(--fsn-color-border-default);
    overflow-y: auto;
    background: var(--fsn-color-bg-surface);
}

.fsd-lenses__list-row {
    display: flex;
    align-items: center;
    padding: 10px 12px;
    cursor: pointer;
    border-bottom: 1px solid var(--fsn-color-border-subtle, rgba(255,255,255,0.05));
    transition: background 100ms;
}
.fsd-lenses__list-row:hover {
    background: var(--fsn-color-bg-elevated);
}
.fsd-lenses__list-row--active {
    background: rgba(6,182,212,0.1);
    border-left: 2px solid var(--fsn-color-primary, #06b6d4);
}

.fsd-lenses__list-row-info {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.fsd-lenses__list-row-name {
    font-size: 13px;
    font-weight: 500;
    color: var(--fsn-color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.fsd-lenses__list-row-query {
    font-size: 11px;
    color: var(--fsn-color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.fsd-lenses__list-row-actions {
    display: flex;
    gap: 2px;
    flex-shrink: 0;
}

.fsd-lenses__detail {
    flex: 1;
    overflow-y: auto;
}

.fsd-lenses__detail-inner {
    padding: 20px;
}

.fsd-lenses__group {
    margin-bottom: 20px;
    border: 1px solid var(--fsn-color-border-default);
    border-radius: 8px;
    overflow: hidden;
}

.fsd-lenses__group-header {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: var(--fsn-color-bg-surface);
    border-bottom: 1px solid var(--fsn-color-border-default);
    font-size: 13px;
    font-weight: 600;
    color: var(--fsn-color-text-primary);
}

.fsd-lenses__group-title {
    flex: 1;
}

.fsd-lenses__item {
    padding: 10px 12px;
    border-bottom: 1px solid var(--fsn-color-border-subtle, rgba(255,255,255,0.05));
}
.fsd-lenses__item:last-child {
    border-bottom: none;
}

.fsd-lenses__item-summary {
    margin: 0;
    font-size: 13px;
    color: var(--fsn-color-text-primary);
}

.fsd-lenses__btn-primary {
    background: var(--fsn-color-primary, #06b6d4);
    color: #fff;
    border: none;
    border-radius: 6px;
    padding: 6px 14px;
    font-size: 13px;
    font-family: inherit;
    cursor: pointer;
}

.fsd-lenses__btn-ghost {
    background: transparent;
    color: var(--fsn-color-text-muted);
    border: 1px solid var(--fsn-color-border-default);
    border-radius: 6px;
    padding: 6px 14px;
    font-size: 13px;
    font-family: inherit;
    cursor: pointer;
}

.fsd-lenses__icon-btn {
    background: transparent;
    border: none;
    color: var(--fsn-color-text-muted);
    cursor: pointer;
    font-size: 13px;
    padding: 3px 6px;
    border-radius: 4px;
}
.fsd-lenses__icon-btn:hover {
    background: var(--fsn-color-bg-elevated);
}
.fsd-lenses__icon-btn--danger:hover {
    color: #ef4444;
    background: rgba(239,68,68,0.15);
}

.fsd-lenses__btn-link {
    background: transparent;
    border: 1px solid var(--fsn-color-primary, #06b6d4);
    color: var(--fsn-color-primary, #06b6d4);
    border-radius: 4px;
    padding: 3px 10px;
    font-size: 12px;
    cursor: pointer;
    font-family: inherit;
}
.fsd-lenses__btn-link:hover {
    background: rgba(6,182,212,0.1);
}
"#;
