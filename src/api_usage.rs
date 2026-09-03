use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::core::{UsageSource, UsageStats};

pub const DEFAULT_URL: &str = "https://api.anthropic.com/api/oauth/usage";

fn default_credentials_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(".claude/.credentials.json")
}

/// Fetches rate-limit utilization from the Anthropic API using the OAuth
/// token stored in the local Claude Code credentials file.
pub struct AnthropicApiUsage {
    credentials_path: PathBuf,
    url: String,
    timeout: Duration,
}

impl Default for AnthropicApiUsage {
    fn default() -> Self {
        Self {
            credentials_path: default_credentials_path(),
            url: DEFAULT_URL.to_string(),
            timeout: Duration::from_secs(10),
        }
    }
}

impl UsageSource for AnthropicApiUsage {
    fn fetch(&self) -> Result<UsageStats> {
        let content = std::fs::read_to_string(&self.credentials_path)
            .with_context(|| format!("reading {}", self.credentials_path.display()))?;
        let credentials: serde_json::Value = serde_json::from_str(&content)?;

        let token = credentials
            .get("claudeAiOauth")
            .and_then(|v| v.get("accessToken"))
            .and_then(|v| v.as_str())
            .context("missing claudeAiOauth.accessToken in credentials file")?;

        let mut response = ureq::get(&self.url)
            .header("Authorization", format!("Bearer {token}"))
            .header("anthropic-beta", "oauth-2025-04-20")
            .header("Accept", "application/json")
            .config()
            .timeout_global(Some(self.timeout))
            .build()
            .call()?;

        let usage: serde_json::Value = response.body_mut().read_json()?;

        let five = usage.get("five_hour").cloned().unwrap_or_default();
        let week = usage.get("seven_day").cloned().unwrap_or_default();

        Ok(UsageStats {
            five_hour_utilization: Some(
                five.get("utilization")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            ),
            five_hour_resets_at: five
                .get("resets_at")
                .and_then(|v| v.as_str())
                .map(String::from),
            seven_day_utilization: Some(
                week.get("utilization")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0),
            ),
            seven_day_resets_at: week
                .get("resets_at")
                .and_then(|v| v.as_str())
                .map(String::from),
            ..Default::default()
        })
    }
}
