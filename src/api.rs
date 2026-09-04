use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};

use crate::core::UsageStats;

pub const DEFAULT_URL: &str = "https://api.anthropic.com/api/oauth/usage";

fn default_credentials_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(".claude/.credentials.json")
}

/// Fetches rate-limit utilization from the Anthropic API using the OAuth
/// token stored in the local Claude Code credentials file.
pub struct Client {
    credentials_path: PathBuf,
    url: String,
    timeout: Duration,
}

impl Default for Client {
    fn default() -> Self {
        Self {
            credentials_path: default_credentials_path(),
            url: DEFAULT_URL.to_string(),
            timeout: Duration::from_secs(10),
        }
    }
}

impl Client {
    pub fn get_usage(&self) -> Result<UsageStats> {
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

        let usage: UsageStats = response.body_mut().read_json()?;
        Ok(usage)
    }
}
