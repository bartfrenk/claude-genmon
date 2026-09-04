mod api;
mod core;

use api::Client;

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
    let client = Client::default();

    match client.get_usage() {
        Ok(stats) => {
            // Match Python's `round()`, which rounds half to even.
            let five_pct = stats.five_hours.utilization.round_ties_even() as i64;
            let week_pct = stats.seven_days.utilization.round_ties_even() as i64;

            // Protect GenMon's bar from unexpected >100% values.
            let five_bar = five_pct.clamp(0, 100);
            let week_bar = week_pct.clamp(0, 100);

            println!("{five_bar}% {week_bar}%");
            println!(
                "<tool>5 hour: {five_pct}% used\n7 day: {week_pct}% used\n5h reset: {}\n7d reset: {}</tool>",
                html_escape(&stats.five_hours.resets_at),
                html_escape(&stats.seven_days.resets_at),
            );
        }
        Err(exc) => output_error(&format!(
            "Claude usage error: no usage data available: {exc:#}"
        )),
    }
}
