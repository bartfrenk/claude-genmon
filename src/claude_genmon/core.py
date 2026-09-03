from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional


@dataclass
class UsageStats:
    five_hour_utilization: Optional[float] = None
    five_hour_resets_at: Optional[str] = None
    seven_day_utilization: Optional[float] = None
    seven_day_resets_at: Optional[str] = None
    total_sessions: Optional[int] = None
    total_messages: Optional[int] = None
    total_tokens: Optional[int] = None


class Usage(ABC):
    @abstractmethod
    def fetch(self) -> Optional[UsageStats]:
        """Return current usage statistics, or None if unavailable."""
