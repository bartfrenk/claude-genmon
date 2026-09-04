mod api;
mod cache;
mod core;

use std::path::PathBuf;

use cache::Cache;

fn icon_path() -> PathBuf {
    let data_dir = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_default()
                .join(".local/share")
        });
    data_dir.join("claude-genmon/anthropic.png")
}

fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _ => out.push(c),
        }
    }
    out
}

/// Truncates an RFC 3339 timestamp (e.g. `2026-09-04T18:40:00.321127+00:00`)
/// to minute precision (`2026-09-04T18:40`). Anything that doesn't look like
/// a timestamp (e.g. `"unknown"`) is returned unchanged.
fn truncate_to_minutes(timestamp: &str) -> &str {
    match timestamp.find('T') {
        Some(t_pos) if timestamp.len() >= t_pos + 6 => &timestamp[..t_pos + 6],
        _ => timestamp,
    }
}

fn output_error(message: &str) {
    println!("<txt>Claude ?</txt>");
    println!("<tool>{}</tool>", html_escape(message));
}

/// Wraps a percentage in Pango markup for GenMon's tooltip: bold always,
/// colored once it gets close to the limit.
fn colored_percent(pct: i64) -> String {
    let text = format!("{pct}%");
    match pct {
        p if p >= 90 => format!("<span foreground=\"#e01b24\"><b>{text}</b></span>"),
        p if p >= 75 => format!("<span foreground=\"#e5a50a\"><b>{text}</b></span>"),
        _ => format!("<b>{text}</b>"),
    }
}

fn main() {
    let icon = icon_path();
    if icon.exists() {
        println!("<img>{}</img>", icon.display());
    }

    let cache = Cache::default();

    match cache.get_usage() {
        Ok(stats) => {
            let five_pct = stats.five_hours.utilization.round_ties_even() as i64;
            let week_pct = stats.seven_days.utilization.round_ties_even() as i64;

            let five_bar = five_pct.clamp(0, 100);
            let week_bar = week_pct.clamp(0, 100);

            println!("<txt><b>5h: {five_bar}% | 7d: {week_bar}%</b></txt>");
            println!(
                "<tool>5 hour: {} used\n7 day: {} used\n5h reset: {}\n7d reset: {}</tool>",
                colored_percent(five_pct),
                colored_percent(week_pct),
                html_escape(truncate_to_minutes(&stats.five_hours.resets_at)),
                html_escape(truncate_to_minutes(&stats.seven_days.resets_at)),
            );
        }
        Err(exc) => output_error(&format!(
            "Claude usage error: no usage data available: {exc:#}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_only_kick_in_near_the_limit() {
        assert_eq!(colored_percent(10), "<b>10%</b>");
        assert_eq!(colored_percent(74), "<b>74%</b>");
        assert_eq!(
            colored_percent(75),
            "<span foreground=\"#e5a50a\"><b>75%</b></span>"
        );
        assert_eq!(
            colored_percent(89),
            "<span foreground=\"#e5a50a\"><b>89%</b></span>"
        );
        assert_eq!(
            colored_percent(90),
            "<span foreground=\"#e01b24\"><b>90%</b></span>"
        );
        assert_eq!(
            colored_percent(120),
            "<span foreground=\"#e01b24\"><b>120%</b></span>"
        );
    }

    #[test]
    fn truncates_timestamps_to_minutes() {
        assert_eq!(
            truncate_to_minutes("2026-09-04T18:40:00.321127+00:00"),
            "2026-09-04T18:40"
        );
        assert_eq!(
            truncate_to_minutes("2026-09-04T18:40:00Z"),
            "2026-09-04T18:40"
        );
        assert_eq!(truncate_to_minutes("2026-09-04T18:40"), "2026-09-04T18:40");
        assert_eq!(truncate_to_minutes("unknown"), "unknown");
        assert_eq!(truncate_to_minutes(""), "");
    }
}
