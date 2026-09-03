#!/usr/bin/env python3

from html import escape

from claude_genmon.api_usage import AnthropicApiUsage


def output_error(message):
    print("<txt>Claude ?</txt>")
    print(f"<tool>{escape(message)}</tool>")


def main():
    usage = AnthropicApiUsage()
    stats = usage.fetch()

    if stats is None:
        output_error(f"Claude usage error: {usage.last_error}")
        return

    five_pct = round(stats.five_hour_utilization or 0)
    week_pct = round(stats.seven_day_utilization or 0)

    # Protect GenMon's bar from unexpected >100% values.
    five_bar = max(0, min(100, five_pct))
    week_bar = max(0, min(100, week_pct))

    print("<txt>Claude</txt>")
    print(f"<bar>{five_bar}</bar>")
    print(f"<bar>{week_bar}</bar>")
    print(
        "<tool>"
        f"5 hour: {five_pct}% used\n"
        f"7 day: {week_pct}% used\n"
        f"5h reset: {escape(str(stats.five_hour_resets_at or 'unknown'))}\n"
        f"7d reset: {escape(str(stats.seven_day_resets_at or 'unknown'))}"
        "</tool>"
    )


if __name__ == "__main__":
    main()
