use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::Context;

use crate::core::UsageStats;

pub const DEFAULT_URL: &str = "https://api.anthropic.com/api/oauth/usage";

fn default_credentials_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default();
    home.join(".claude/.credentials.json")
}

/// A source of usage statistics, fallible in a way that distinguishes
/// "rate limited" (recoverable, carries a `Retry-After`) from other errors.
pub trait UsageProvider {
    fn get_usage(&self) -> Result<UsageStats, GetUsageError>;
}

/// Error returned by [`UsageProvider::get_usage`].
#[derive(Debug)]
pub enum GetUsageError {
    /// The API responded 429, optionally with a `Retry-After` header value.
    RateLimited {
        retry_after: Option<String>,
    },
    Other(anyhow::Error),
}

impl fmt::Display for GetUsageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GetUsageError::RateLimited {
                retry_after: Some(v),
            } => {
                write!(f, "rate limited, retry after {v}")
            }
            GetUsageError::RateLimited { retry_after: None } => write!(f, "rate limited"),
            GetUsageError::Other(e) => write!(f, "{e:#}"),
        }
    }
}

impl std::error::Error for GetUsageError {}

impl From<anyhow::Error> for GetUsageError {
    fn from(e: anyhow::Error) -> Self {
        GetUsageError::Other(e)
    }
}

impl From<serde_json::Error> for GetUsageError {
    fn from(e: serde_json::Error) -> Self {
        GetUsageError::Other(e.into())
    }
}

impl From<ureq::Error> for GetUsageError {
    fn from(e: ureq::Error) -> Self {
        GetUsageError::Other(e.into())
    }
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

impl UsageProvider for Client {
    fn get_usage(&self) -> Result<UsageStats, GetUsageError> {
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
            .http_status_as_error(false)
            .build()
            .call()?;

        if response.status() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .map(String::from);
            return Err(GetUsageError::RateLimited { retry_after });
        }

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("http status: {}", response.status()).into());
        }

        let usage: UsageStats = response.body_mut().read_json()?;
        Ok(usage)
    }
}
