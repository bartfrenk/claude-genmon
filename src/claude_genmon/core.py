from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass


@dataclass
class UsageStats:
    five_hour_utilization: float | None = None
    five_hour_resets_at: str | None = None
    seven_day_utilization: float | None = None
    seven_day_resets_at: str | None = None
    total_sessions: int | None = None
    total_messages: int | None = None
    total_tokens: int | None = None


class Usage(ABC):
    @abstractmethod
    def fetch(self) -> UsageStats | None:
        """Return current usage statistics, or None if unavailable."""
