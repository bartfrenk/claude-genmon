from __future__ import annotations

from abc import ABC, abstractmethod
from dataclasses import dataclass, fields


@dataclass
class UsageStats:
    five_hour_utilization: float | None = None
    five_hour_resets_at: str | None = None
    seven_day_utilization: float | None = None
    seven_day_resets_at: str | None = None
    total_sessions: int | None = None
    total_messages: int | None = None
    total_tokens: int | None = None

    def __or__(self, other: UsageStats) -> UsageStats:
        """Left-biased merge: each field takes this instance's value, falling
        back to `other`'s when this instance's is None."""
        return UsageStats(
            **{  # pyright: ignore[reportAny]
                f.name: (
                    getattr(self, f.name)
                    if getattr(self, f.name) is not None
                    else getattr(other, f.name)
                )
                for f in fields(self)
            }
        )


class Usage(ABC):
    @abstractmethod
    def fetch(self) -> UsageStats | None:
        """Return current usage statistics, or None if unavailable."""
