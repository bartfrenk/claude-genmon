mod api_usage;
mod core;
mod local_usage;

use api_usage::AnthropicApiUsage;
use local_usage::LocalDiskUsage;

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

fn output_error(message: &str) {
    println!("<txt>Claude ?</txt>");
    println!("<tool>{}</tool>", html_escape(message));
}

fn main() {
    let api = AnthropicApiUsage::default();
    let local = LocalDiskUsage::default();

    match core::combine(&api, &local) {
        Ok(stats) => {
            // Match Python's `round()`, which rounds half to even.
            let five_pct = stats.five_hour_utilization.unwrap_or(0.0).round_ties_even() as i64;
            let week_pct = stats.seven_day_utilization.unwrap_or(0.0).round_ties_even() as i64;

            // Protect GenMon's bar from unexpected >100% values.
            let five_bar = five_pct.clamp(0, 100);
            let week_bar = week_pct.clamp(0, 100);

            println!("{five_bar}% {week_bar}%");
            println!(
                "<tool>5 hour: {five_pct}% used\n7 day: {week_pct}% used\n5h reset: {}\n7d reset: {}</tool>",
                html_escape(stats.five_hour_resets_at.as_deref().unwrap_or("unknown")),
                html_escape(stats.seven_day_resets_at.as_deref().unwrap_or("unknown")),
            );
        }
        Err(exc) => output_error(&format!(
            "Claude usage error: no usage data available: {exc:#}"
        )),
    }
}
