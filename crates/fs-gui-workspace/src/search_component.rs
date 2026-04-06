// search_component.rs — SearchComponent: global search across all programs.
//
// Design Pattern: Strategy (SearchStrategy — per-source search backends)
//   The component renders a unified input field + expandable filter area.
//   Each active search strategy (local, bus, program-scoped) returns LensItems
//   grouped by source.  Strategies are injected via ComponentCtx.config.
//
// Data source: fs-lenses / fs-bus via gRPC (Sandbox O7).
// Writes: Bus-Events ("search.query" topic).

use fs_render::component::{ButtonStyle, ComponentCtx, ComponentTrait, LayoutElement, TextSize};
use fs_render::layout::SlotKind;

// ── SearchFilter ──────────────────────────────────────────────────────────────

/// Active search scope narrowing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
enum SearchFilter {
    /// No filter — search everything.
    All,
    /// Filter by a tag (e.g. `"files"`, `"contacts"`).
    ByTag(String),
    /// Search within a specific program (e.g. `"fs-store"`).
    InProgram(String),
    /// Search within a named program group.
    InGroup(String),
    /// Search across all programs that support the query.
    CrossProgram,
}

impl SearchFilter {
    fn label_key(&self) -> String {
        match self {
            Self::All => "search-filter-all".into(),
            Self::ByTag(tag) => format!("search-filter-tag:{tag}"),
            Self::InProgram(id) => format!("search-filter-program:{id}"),
            Self::InGroup(name) => format!("search-filter-group:{name}"),
            Self::CrossProgram => "search-filter-cross-program".into(),
        }
    }

    fn action(&self) -> String {
        match self {
            Self::All => "search.filter=all".into(),
            Self::ByTag(tag) => format!("search.filter=tag:{tag}"),
            Self::InProgram(id) => format!("search.filter=program:{id}"),
            Self::InGroup(name) => format!("search.filter=group:{name}"),
            Self::CrossProgram => "search.filter=cross".into(),
        }
    }
}

// ── SearchResult stub ─────────────────────────────────────────────────────────

/// A single search result from one strategy/source.
#[derive(Debug, Clone)]
struct SearchResultEntry {
    icon_key: String,
    label: String,
    source: String,
    action: String,
}

impl SearchResultEntry {
    fn as_element(&self) -> LayoutElement {
        LayoutElement::SearchResult {
            icon_key: self.icon_key.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            action: self.action.clone(),
        }
    }
}

// ── SearchComponent ───────────────────────────────────────────────────────────

/// Global search bar with expandable filter area and grouped results.
///
/// # Wiring (ComponentCtx.config)
///
/// | key | value |
/// |-----|-------|
/// | `"query"` | Current search input (empty = no results shown) |
/// | `"expanded"` | `"true"` when the advanced filter area is open |
/// | `"filter"` | `"all"` \| `"tag:{tag}"` \| `"program:{id}"` \| `"group:{name}"` \| `"cross"` |
/// | `"results_json"` | JSON-serialised result list (optional, shell-injected) |
///
/// In production the shell injects live results from the search Bus topic.
pub struct SearchComponent {
    id: &'static str,
}

impl SearchComponent {
    /// Create a new search component.
    #[must_use]
    pub fn new() -> Self {
        Self { id: "search" }
    }

    fn filter_buttons() -> Vec<LayoutElement> {
        let filters = [
            SearchFilter::All,
            SearchFilter::ByTag("files".into()),
            SearchFilter::ByTag("contacts".into()),
            SearchFilter::InProgram("fs-store".into()),
            SearchFilter::CrossProgram,
        ];
        filters
            .iter()
            .map(|f| LayoutElement::Button {
                label_key: f.label_key(),
                action: f.action(),
                style: ButtonStyle::Ghost,
            })
            .collect()
    }

    fn stub_results() -> Vec<SearchResultEntry> {
        // Placeholder results — replaced by live Bus-aggregated data.
        vec![
            SearchResultEntry {
                icon_key: "fs:apps/store".into(),
                label: "search-result-placeholder-store".into(),
                source: "fs-store".into(),
                action: "search.open:store.result-1".into(),
            },
            SearchResultEntry {
                icon_key: "fs:apps/wiki".into(),
                label: "search-result-placeholder-wiki".into(),
                source: "wiki.team-a".into(),
                action: "search.open:wiki.result-1".into(),
            },
        ]
    }

    fn results_by_source(results: &[SearchResultEntry]) -> Vec<LayoutElement> {
        // Group results by source, each group as a separator + list.
        let mut elements: Vec<LayoutElement> = Vec::new();
        let mut seen_sources: Vec<String> = Vec::new();

        for result in results {
            if !seen_sources.contains(&result.source) {
                seen_sources.push(result.source.clone());
                elements.push(LayoutElement::Separator {
                    label_key: Some(result.source.clone()),
                });
            }
            elements.push(result.as_element());
        }

        elements
    }
}

impl Default for SearchComponent {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentTrait for SearchComponent {
    fn component_id(&self) -> &str {
        self.id
    }

    fn name_key(&self) -> &'static str {
        "component-search-name"
    }

    fn description_key(&self) -> &'static str {
        "component-search-desc"
    }

    fn slot_preference(&self) -> SlotKind {
        SlotKind::Fill
    }

    fn min_width(&self) -> u32 {
        240
    }

    fn render(&self, ctx: &ComponentCtx) -> Vec<LayoutElement> {
        let query = ctx.config.get("query").cloned().unwrap_or_default();
        let expanded = ctx.config.get("expanded").map(String::as_str) == Some("true");

        let mut elements = vec![
            // Main search input
            LayoutElement::TextInput {
                placeholder_key: "search-input-placeholder".into(),
                value: query.clone(),
                on_change_action: "search.query_changed".into(),
            },
            // Expand/collapse advanced filters
            LayoutElement::Button {
                label_key: if expanded {
                    "search-filter-collapse".into()
                } else {
                    "search-filter-expand".into()
                },
                action: "search.toggle_expanded".into(),
                style: ButtonStyle::Ghost,
            },
        ];

        // Advanced filter panel
        if expanded {
            elements.push(LayoutElement::Separator {
                label_key: Some("search-filter-section".into()),
            });
            let filter_btns = Self::filter_buttons();
            elements.push(LayoutElement::Row {
                children: filter_btns,
                gap: 6,
            });
        }

        // Results (only when query is non-empty)
        if !query.is_empty() {
            elements.push(LayoutElement::Separator {
                label_key: Some("search-results-section".into()),
            });
            let results = Self::stub_results();
            if results.is_empty() {
                elements.push(LayoutElement::Text {
                    content: "search-no-results".into(),
                    size: TextSize::Body,
                    color: None,
                });
            } else {
                elements.extend(Self::results_by_source(&results));
            }
        }

        elements
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use fs_render::layout::{ShellKind, SlotKind};

    #[test]
    fn component_id() {
        let c = SearchComponent::new();
        assert_eq!(c.component_id(), "search");
    }

    #[test]
    fn slot_preference_is_fill() {
        let c = SearchComponent::new();
        assert_eq!(c.slot_preference(), SlotKind::Fill);
    }

    #[test]
    fn render_empty_query_no_results() {
        let c = SearchComponent::new();
        let ctx = ComponentCtx::test(ShellKind::Main, SlotKind::Fill);
        let els = c.render(&ctx);
        let has_result = els
            .iter()
            .any(|e| matches!(e, LayoutElement::SearchResult { .. }));
        assert!(!has_result);
    }

    #[test]
    fn render_with_query_shows_results() {
        let c = SearchComponent::new();
        let mut ctx = ComponentCtx::test(ShellKind::Main, SlotKind::Fill);
        ctx.config.insert("query".into(), "wiki".into());
        let els = c.render(&ctx);
        let has_result = els
            .iter()
            .any(|e| matches!(e, LayoutElement::SearchResult { .. }));
        assert!(has_result);
    }

    #[test]
    fn render_collapsed_no_filter_row() {
        let c = SearchComponent::new();
        let ctx = ComponentCtx::test(ShellKind::Main, SlotKind::Fill);
        let els = c.render(&ctx);
        let has_filter_row = els
            .iter()
            .any(|e| matches!(e, LayoutElement::Row { children, .. } if children.len() > 2));
        assert!(!has_filter_row);
    }

    #[test]
    fn render_expanded_shows_filter_row() {
        let c = SearchComponent::new();
        let mut ctx = ComponentCtx::test(ShellKind::Main, SlotKind::Fill);
        ctx.config.insert("expanded".into(), "true".into());
        let els = c.render(&ctx);
        let has_filter_row = els
            .iter()
            .any(|e| matches!(e, LayoutElement::Row { children, .. } if !children.is_empty()));
        assert!(has_filter_row);
    }

    #[test]
    fn results_grouped_by_source_has_separators() {
        let results = SearchComponent::stub_results();
        let grouped = SearchComponent::results_by_source(&results);
        let sep_count = grouped
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    LayoutElement::Separator {
                        label_key: Some(_),
                        ..
                    }
                )
            })
            .count();
        // Two distinct sources → two separators
        assert_eq!(sep_count, 2);
    }

    #[test]
    fn search_filter_all_action() {
        let f = SearchFilter::All;
        assert_eq!(f.action(), "search.filter=all");
    }

    #[test]
    fn search_filter_tag_label_contains_tag() {
        let f = SearchFilter::ByTag("files".into());
        assert!(f.label_key().contains("files"));
    }
}
