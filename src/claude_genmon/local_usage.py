from __future__ import annotations

import json
import os

from claude_genmon.core import Usage, UsageStats

DEFAULT_STATS_CACHE_PATH = os.path.expanduser("~/.claude/stats-cache.json")

TOKEN_FIELDS = (
    "inputTokens",
    "outputTokens",
    "cacheReadInputTokens",
    "cacheCreationInputTokens",
)


class LocalDiskUsage(Usage):
    """Reads cumulative usage stats straight from Claude Code's local
    stats cache, with no network call."""

    def __init__(self, stats_cache_path: str = DEFAULT_STATS_CACHE_PATH):
        super().__init__()
        self.stats_cache_path = stats_cache_path

    def _fetch(self) -> UsageStats | None:
        with open(self.stats_cache_path, "r", encoding="utf-8") as f:
            cache = json.load(f)

        total_tokens = sum(
            model_stats.get(field, 0)
            for model_stats in (cache.get("modelUsage") or {}).values()
            for field in TOKEN_FIELDS
        )

        return UsageStats(
            total_sessions=cache.get("totalSessions"),
            total_messages=cache.get("totalMessages"),
            total_tokens=total_tokens,
        )
