use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::core::{UsageSource, UsageStats};

const TOKEN_FIELDS: [&str; 4] = [
    "inputTokens",
    "outputTokens",
    "cacheReadInputTokens",
    "cacheCreationInputTokens",
];

fn default_stats_cache_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(".claude/stats-cache.json")
}

/// Reads cumulative usage stats straight from Claude Code's local stats
/// cache, with no network call.
pub struct LocalDiskUsage {
    stats_cache_path: PathBuf,
}

impl Default for LocalDiskUsage {
    fn default() -> Self {
        Self {
            stats_cache_path: default_stats_cache_path(),
        }
    }
}

impl UsageSource for LocalDiskUsage {
    fn fetch(&self) -> Result<UsageStats> {
        let content = std::fs::read_to_string(&self.stats_cache_path)
            .with_context(|| format!("reading {}", self.stats_cache_path.display()))?;
        let cache: serde_json::Value = serde_json::from_str(&content)?;

        let total_tokens: u64 = cache
            .get("modelUsage")
            .and_then(|v| v.as_object())
            .map(|model_usage| {
                model_usage
                    .values()
                    .map(|model_stats| {
                        TOKEN_FIELDS
                            .iter()
                            .map(|field| {
                                model_stats
                                    .get(*field)
                                    .and_then(|v| v.as_u64())
                                    .unwrap_or(0)
                            })
                            .sum::<u64>()
                    })
                    .sum()
            })
            .unwrap_or(0);

        Ok(UsageStats {
            total_sessions: cache.get("totalSessions").and_then(|v| v.as_u64()),
            total_messages: cache.get("totalMessages").and_then(|v| v.as_u64()),
            total_tokens: Some(total_tokens),
            ..Default::default()
        })
    }
}
