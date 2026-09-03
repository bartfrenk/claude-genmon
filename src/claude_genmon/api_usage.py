from __future__ import annotations

import json
import os
import urllib.request
from typing import Optional

from claude_genmon.core import Usage, UsageStats

DEFAULT_CREDENTIALS_PATH = os.path.expanduser("~/.claude/.credentials.json")
DEFAULT_URL = "https://api.anthropic.com/api/oauth/usage"


class AnthropicApiUsage(Usage):
    """Fetches rate-limit utilization from the Anthropic API using the
    OAuth token stored in the local Claude Code credentials file."""

    def __init__(
        self,
        credentials_path: str = DEFAULT_CREDENTIALS_PATH,
        url: str = DEFAULT_URL,
        timeout: int = 10,
    ):
        self.credentials_path = credentials_path
        self.url = url
        self.timeout = timeout
        self.last_error: Optional[Exception] = None

    def fetch(self) -> Optional[UsageStats]:
        self.last_error = None
        try:
            with open(self.credentials_path, "r", encoding="utf-8") as f:
                credentials = json.load(f)

            token = credentials["claudeAiOauth"]["accessToken"]

            request = urllib.request.Request(
                self.url,
                headers={
                    "Authorization": f"Bearer {token}",
                    "anthropic-beta": "oauth-2025-04-20",
                    "Accept": "application/json",
                },
            )

            with urllib.request.urlopen(request, timeout=self.timeout) as response:
                usage = json.load(response)

            five = usage.get("five_hour") or {}
            week = usage.get("seven_day") or {}

            return UsageStats(
                five_hour_utilization=float(five.get("utilization", 0)),
                five_hour_resets_at=five.get("resets_at"),
                seven_day_utilization=float(week.get("utilization", 0)),
                seven_day_resets_at=week.get("resets_at"),
            )
        except Exception as exc:
            self.last_error = exc
            return None
