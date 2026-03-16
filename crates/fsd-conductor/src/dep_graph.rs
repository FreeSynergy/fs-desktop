/// Dependency graph — SVG visualisation of FSN service states.
///
/// Lists all fsn-*.service units via systemctl and renders them as a grid.
/// Dependency edges (fsn.requires labels) require Podman container labels
/// which are not available via systemctl — edge rendering is omitted.
use dioxus::prelude::*;
use fsn_container::{SystemctlManager, UnitActiveState};

use crate::service_list::list_fsn_units;

// ── Layout constants ──────────────────────────────────────────────────────────

const NODE_W: f64 = 140.0;
const NODE_H: f64 = 40.0;
const H_GAP: f64  = 60.0;
const V_GAP: f64  = 80.0;
const COLS: usize  = 4;

// ── GraphNode ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct GraphNode {
    name:   String,
    health: &'static str, // "ok" | "warn" | "err" | "unknown"
    x: f64,
    y: f64,
}

// ── DependencyGraph ───────────────────────────────────────────────────────────

/// SVG service-graph component — shows all FSN units with their active state.
#[component]
pub fn DependencyGraph() -> Element {
    let mut nodes: Signal<Vec<GraphNode>> = use_signal(Vec::new);
    let mut error: Signal<Option<String>> = use_signal(|| None);

    // Poll every 5 seconds
    use_future(move || async move {
        let mgr = SystemctlManager::user();
        loop {
            let units = list_fsn_units().await;
            if units.is_empty() {
                error.set(Some("No FSN services found.".into()));
            } else {
                let mut result = Vec::new();
                for (i, unit) in units.iter().enumerate() {
                    let health = match mgr.service_status(unit).await {
                        Ok(s) => match s.active_state {
                            UnitActiveState::Active       => "ok",
                            UnitActiveState::Activating   => "warn",
                            UnitActiveState::Failed       => "err",
                            _                             => "unknown",
                        },
                        Err(_) => "unknown",
                    };
                    let col = i % COLS;
                    let row = i / COLS;
                    result.push(GraphNode {
                        name:   unit.clone(),
                        health,
                        x: H_GAP + (col as f64) * (NODE_W + H_GAP),
                        y: V_GAP + (row as f64) * (NODE_H + V_GAP),
                    });
                }
                nodes.set(result);
                error.set(None);
            }
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    let list = nodes.read().clone();
    let svg_w = (COLS as f64) * (NODE_W + H_GAP) + H_GAP;
    let svg_h = if list.is_empty() { 120.0 } else {
        let rows = (list.len() as f64 / COLS as f64).ceil();
        rows * (NODE_H + V_GAP) + V_GAP
    };

    rsx! {
        div {
            class: "fsd-dep-graph",

            // Header
            div {
                style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 16px;",
                h2 { style: "margin: 0; font-size: 18px;", "Service Graph" }
                span {
                    style: "font-size: 12px; color: var(--fsn-text-muted);",
                    "FSN systemd units"
                }
            }

            // Error
            if let Some(err) = error.read().as_deref() {
                div {
                    style: "color: var(--fsn-warning); font-size: 13px; margin-bottom: 12px;",
                    "{err}"
                }
            }

            if list.is_empty() {
                div {
                    style: "text-align: center; color: var(--fsn-text-muted); padding: 48px;",
                    "No FSN services found."
                }
            } else {
                // SVG canvas
                div {
                    style: "overflow: auto;",
                    dangerous_inner_html: "{build_svg(&list, svg_w, svg_h)}"
                }

                // Legend
                div {
                    style: "display: flex; gap: 16px; margin-top: 12px; font-size: 12px; color: var(--fsn-text-muted);",
                    LegendDot { color: "#34d399", label: "Active" }
                    LegendDot { color: "#fbbf24", label: "Starting" }
                    LegendDot { color: "#f87171", label: "Failed" }
                    LegendDot { color: "#5a6e88", label: "Inactive" }
                }
            }
        }
    }
}

#[component]
fn LegendDot(color: &'static str, label: &'static str) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 4px;",
            div { style: "width: 10px; height: 10px; border-radius: 50%; background: {color};" }
            span { "{label}" }
        }
    }
}

// ── SVG renderer ──────────────────────────────────────────────────────────────

fn health_color(h: &str) -> &'static str {
    match h {
        "ok"   => "#34d399",
        "warn" => "#fbbf24",
        "err"  => "#f87171",
        _      => "#5a6e88",
    }
}

fn build_svg(nodes: &[GraphNode], w: f64, h: f64) -> String {
    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" style="display:block;">"#,
    );

    for node in nodes {
        let x      = node.x;
        let y      = node.y;
        let cx     = x + NODE_W / 2.0;
        let cy     = y + NODE_H / 2.0;
        let dot_cx = x + 14.0;
        let text_y = cy + 4.5;
        let color  = health_color(node.health);
        let label: String = if node.name.len() > 18 {
            format!("{}…", &node.name[..17])
        } else {
            node.name.clone()
        };

        svg.push_str(&format!(
            r##"<rect x="{x:.0}" y="{y:.0}" width="{NODE_W}" height="{NODE_H}" rx="6" fill="#162032" stroke="{color}" stroke-width="2"/>"##
        ));
        svg.push_str(&format!(
            r##"<circle cx="{dot_cx:.0}" cy="{cy:.0}" r="5" fill="{color}"/>"##
        ));
        svg.push_str(&format!(
            r##"<text x="{cx:.0}" y="{text_y:.0}" text-anchor="middle" fill="#e8edf5" font-size="12" font-family="monospace">{label}</text>"##
        ));
    }

    svg.push_str("</svg>");
    svg
}
