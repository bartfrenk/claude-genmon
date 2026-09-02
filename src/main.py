#!/usr/bin/env python3

import json
import os
import urllib.request
from html import escape

CREDS = os.path.expanduser("~/.claude/.credentials.json")
URL = "https://api.anthropic.com/api/oauth/usage"


def output_error(message):
    print("<txt>Claude ?</txt>")
    print(f"<tool>{escape(message)}</tool>")


try:
    with open(CREDS, "r", encoding="utf-8") as f:
        credentials = json.load(f)

    token = credentials["claudeAiOauth"]["accessToken"]

    request = urllib.request.Request(
        URL,
        headers={
            "Authorization": f"Bearer {token}",
            "anthropic-beta": "oauth-2025-04-20",
            "Accept": "application/json",
        },
    )

    with urllib.request.urlopen(request, timeout=10) as response:
        usage = json.load(response)

    five = usage.get("five_hour") or {}
    week = usage.get("seven_day") or {}

    five_pct = round(float(five.get("utilization", 0)))
    week_pct = round(float(week.get("utilization", 0)))

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
        f"5h reset: {escape(str(five.get('resets_at', 'unknown')))}\n"
        f"7d reset: {escape(str(week.get('resets_at', 'unknown')))}"
        "</tool>"
    )

except Exception as exc:
    output_error(f"Claude usage error: {exc}")
