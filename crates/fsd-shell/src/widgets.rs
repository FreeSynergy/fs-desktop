/// Desktop widgets — standalone UI cards that can be placed on the desktop
/// or embedded in any layout.
///
/// - `ClockWidget` — analog/digital clock with second-accurate updates.
/// - `SystemInfoWidget` — hostname, uptime, memory and disk at a glance.
use chrono::Local;
use dioxus::prelude::*;

// ── ClockWidget ───────────────────────────────────────────────────────────────

/// A clock widget that updates every second.
///
/// Displays the current time (HH:MM:SS) and date (Weekday, DD Month YYYY).
#[component]
pub fn ClockWidget() -> Element {
    let mut time_str = use_signal(|| Local::now().format("%H:%M:%S").to_string());
    let mut date_str = use_signal(|| Local::now().format("%A, %d %B %Y").to_string());

    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            time_str.set(Local::now().format("%H:%M:%S").to_string());
            date_str.set(Local::now().format("%A, %d %B %Y").to_string());
        }
    });

    rsx! {
        div {
            class: "fsn-widget fsn-widget--clock",
            style: "background: var(--fsn-color-bg-surface); \
                    border: 1px solid var(--fsn-color-border-default); \
                    border-radius: var(--fsn-radius-lg); \
                    padding: 20px 24px; \
                    display: flex; flex-direction: column; align-items: center; gap: 6px; \
                    min-width: 200px;",

            span {
                style: "font-size: 36px; font-weight: 700; letter-spacing: 2px; \
                        font-variant-numeric: tabular-nums; \
                        color: var(--fsn-color-primary);",
                "{time_str}"
            }
            span {
                style: "font-size: 13px; color: var(--fsn-color-text-muted);",
                "{date_str}"
            }
        }
    }
}

// ── SystemInfoWidget ──────────────────────────────────────────────────────────

/// Snapshot of system information.
#[derive(Clone, Default)]
struct SysInfo {
    hostname: String,
    uptime:   String,
    mem_used: String,
    mem_total: String,
    disk_used: String,
    disk_total: String,
}

/// A system-info widget showing hostname, uptime, memory and disk.
///
/// Reads `/etc/hostname`, `/proc/uptime`, `/proc/meminfo` and uses `df -h /`
/// for disk information. Refreshes every 10 seconds.
#[component]
pub fn SystemInfoWidget() -> Element {
    let mut info = use_signal(SysInfo::default);

    use_future(move || async move {
        loop {
            let snapshot = tokio::task::spawn_blocking(read_sys_info).await.unwrap_or_default();
            info.set(snapshot);
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
        }
    });

    let i = info.read();
    rsx! {
        div {
            class: "fsn-widget fsn-widget--sysinfo",
            style: "background: var(--fsn-color-bg-surface); \
                    border: 1px solid var(--fsn-color-border-default); \
                    border-radius: var(--fsn-radius-lg); \
                    padding: 16px 20px; \
                    display: flex; flex-direction: column; gap: 10px; \
                    min-width: 240px;",

            // Widget title
            div {
                style: "font-size: 12px; font-weight: 600; text-transform: uppercase; \
                        letter-spacing: 0.08em; color: var(--fsn-color-text-muted); \
                        border-bottom: 1px solid var(--fsn-color-border-default); \
                        padding-bottom: 8px;",
                "System Info"
            }

            SysRow { icon: "🖥",  label: "Host",   value: i.hostname.clone() }
            SysRow { icon: "⏱",  label: "Uptime", value: i.uptime.clone() }
            SysRow { icon: "🧠",  label: "Memory", value: format!("{} / {}", i.mem_used, i.mem_total) }
            SysRow { icon: "💾",  label: "Disk",   value: format!("{} / {}", i.disk_used, i.disk_total) }
        }
    }
}

// ── SysRow ────────────────────────────────────────────────────────────────────

#[component]
fn SysRow(icon: String, label: String, value: String) -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 10px; font-size: 13px;",
            span { style: "font-size: 16px; min-width: 20px;", "{icon}" }
            span {
                style: "color: var(--fsn-color-text-muted); min-width: 56px;",
                "{label}"
            }
            span {
                style: "color: var(--fsn-color-text-primary); font-weight: 500; \
                        overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
                if value.is_empty() { "—" } else { "{value}" }
            }
        }
    }
}

// ── system reads ─────────────────────────────────────────────────────────────

fn read_sys_info() -> SysInfo {
    SysInfo {
        hostname:   read_hostname(),
        uptime:     read_uptime(),
        mem_used:   read_mem_used(),
        mem_total:  read_mem_total(),
        disk_used:  read_disk_used(),
        disk_total: read_disk_total(),
    }
}

fn read_hostname() -> String {
    std::fs::read_to_string("/etc/hostname")
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn read_uptime() -> String {
    let raw = std::fs::read_to_string("/proc/uptime").unwrap_or_default();
    let secs: f64 = raw
        .split_whitespace()
        .next()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let secs = secs as u64;
    let days  = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let mins  = (secs % 3600) / 60;
    if days > 0 {
        format!("{days}d {hours}h {mins}m")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else {
        format!("{mins}m")
    }
}

fn parse_meminfo_kb(key: &str) -> u64 {
    let raw = std::fs::read_to_string("/proc/meminfo").unwrap_or_default();
    raw.lines()
        .find(|l| l.starts_with(key))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

fn kb_to_display(kb: u64) -> String {
    if kb >= 1_048_576 {
        format!("{:.1}G", kb as f64 / 1_048_576.0)
    } else if kb >= 1024 {
        format!("{:.0}M", kb as f64 / 1024.0)
    } else {
        format!("{kb}K")
    }
}

fn read_mem_total() -> String {
    kb_to_display(parse_meminfo_kb("MemTotal:"))
}

fn read_mem_used() -> String {
    let total     = parse_meminfo_kb("MemTotal:");
    let available = parse_meminfo_kb("MemAvailable:");
    kb_to_display(total.saturating_sub(available))
}

fn read_disk_used() -> String {
    disk_stat(true)
}

fn read_disk_total() -> String {
    disk_stat(false)
}

/// Returns used or total disk space for `/` via `df`.
fn disk_stat(used: bool) -> String {
    let out = std::process::Command::new("df")
        .args(["--output=used,size", "-k", "/"])
        .output();
    let Ok(out) = out else { return "?".into() };
    let text = String::from_utf8_lossy(&out.stdout);
    // second line: "used size"
    let mut lines = text.lines();
    let _ = lines.next(); // header
    let data = lines.next().unwrap_or("");
    let mut parts = data.split_whitespace();
    let used_kb:  u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    let total_kb: u64 = parts.next().and_then(|v| v.parse().ok()).unwrap_or(0);
    if used { kb_to_display(used_kb) } else { kb_to_display(total_kb) }
}
